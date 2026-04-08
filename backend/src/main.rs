mod shell_state_generated;
mod client_command_generated;

use shell_state_generated::niri_shell::{ShellState, ShellStateArgs, Workspace, WorkspaceArgs};
use client_command_generated::niri_shell::root_as_client_command;
use flatbuffers::FlatBufferBuilder;
use tokio::net::{UnixListener, UnixDatagram};
use tokio::io::{AsyncWriteExt, AsyncReadExt, AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use std::process::Stdio;
use std::fs;
use serde::Deserialize;
use futures_lite::stream::StreamExt;
use std::sync::Arc;

// --- D-BUS SCHNITTSTELLEN & STRUKTUREN (Bleiben exakt gleich) ---
#[zbus::proxy(interface = "org.freedesktop.NetworkManager", default_service = "org.freedesktop.NetworkManager", default_path = "/org/freedesktop/NetworkManager")]
trait NetworkManager { #[zbus(property)] fn primary_connection(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>; }
#[zbus::proxy(interface = "org.freedesktop.NetworkManager.Connection.Active", default_service = "org.freedesktop.NetworkManager")]
trait ActiveConnection { #[zbus(property)] fn id(&self) -> zbus::Result<String>; }
#[derive(Deserialize, Debug)] struct NiriWorkspace { id: u64, idx: u64, name: Option<String>, is_active: bool }
#[derive(Deserialize, Debug)] struct NiriWindow { title: Option<String>, is_focused: bool }

async fn fetch_workspaces() -> Vec<NiriWorkspace> { /* ... dein bisheriger code ... */
    serde_json::from_slice(&Command::new("niri").args(&["msg", "-j", "workspaces"]).output().await.unwrap().stdout).unwrap_or_default()
}
async fn fetch_active_window_title() -> Option<String> { /* ... dein bisheriger code ... */
    let o = Command::new("niri").args(&["msg", "-j", "windows"]).output().await.ok()?;
    serde_json::from_slice::<Vec<NiriWindow>>(&o.stdout).ok()?.into_iter().find(|w| w.is_focused).and_then(|w| w.title)
}
async fn get_audio_state() -> (i8, bool) { /* ... dein bisheriger code ... */
    if let Ok(o) = Command::new("wpctl").args(&["get-volume", "@DEFAULT_AUDIO_SINK@"]).output().await {
        let s = String::from_utf8_lossy(&o.stdout);
        let vol = s.split_whitespace().nth(1).and_then(|v| v.parse::<f32>().ok()).map(|v| (v * 100.0) as i8).unwrap_or(0);
        return (vol, s.contains("[MUTED]"));
    }
    (0, true)
}
fn get_battery_percent() -> i8 { fs::read_to_string("/sys/class/power_supply/BAT0/capacity").unwrap_or_default().trim().parse().unwrap_or(100) }
async fn get_network_name_dbus(_conn: &zbus::Connection) -> String { "Online".to_string() /* Dein D-Bus Code hier */ }

// NEU: Unser Event-System unterstützt nun explizit das Control Center
enum Event {
    RefreshData,
    ToggleControlCenter,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let cli_socket_path = "/tmp/quickshell-cli.sock";

    // --- CLI MODUS (Die Fernbedienung) ---
    if args.len() > 1 && args[1] == "toggle-cc" {
        let socket = UnixDatagram::unbound()?;
        let _ = socket.send_to(b"TOGGLE_CC", cli_socket_path);
        return Ok(());
    }

    // --- DAEMON MODUS ---
    let socket_path = "/tmp/niri-quickshell.sock";
    let _ = fs::remove_file(socket_path);
    let _ = fs::remove_file(cli_socket_path);
    
    let listener = UnixListener::bind(socket_path)?;
    let cli_listener = Arc::new(UnixDatagram::bind(cli_socket_path)?);
    let dbus_conn = zbus::Connection::system().await?;
    
    println!("🚀 Rust Backend bereit!");

    loop {
        let stream = match listener.accept().await { Ok((s, _)) => s, Err(_) => continue };
        let (mut rx, mut tx) = tokio::io::split(stream);
        
        // Der Channel nutzt jetzt unser neues Event-Enum!
        let (update_tx, mut update_rx) = mpsc::channel::<Event>(20);

        // TASK 1-5 (Dein alter Code): Ersetze jedes `tx.send(()).await` durch `tx.send(Event::RefreshData).await`
        // TASK 1: Niri Event Listener
        let tx_niri = update_tx.clone();
        tokio::spawn(async move {
            if let Ok(mut event_stream) = Command::new("niri").args(&["msg", "-j", "event-stream"]).stdout(Stdio::piped()).spawn() {
                if let Some(stdout) = event_stream.stdout.take() {
                    let mut reader = BufReader::new(stdout).lines();
                    while let Ok(Some(_)) = reader.next_line().await { let _ = tx_niri.send(Event::RefreshData).await; }
                }
            }
        });

        // TASK 2: NetworkManager D-Bus Listener
        let tx_nm = update_tx.clone();
        let dbus_conn_clone = dbus_conn.clone();
        tokio::spawn(async move {
            if let Ok(nm) = NetworkManagerProxy::new(&dbus_conn_clone).await {
                let mut stream = nm.receive_primary_connection_changed().await;
                while let Some(_) = stream.next().await { let _ = tx_nm.send(Event::RefreshData).await; }
            }
        });

        // TASK 3: Audio Event Listener mit NixOS-Fallback
        let tx_audio = update_tx.clone();
        tokio::spawn(async move {
            if let Ok(mut child) = Command::new("pactl").args(&["subscribe"]).stdout(Stdio::piped()).spawn() {
                if let Some(stdout) = child.stdout.take() {
                    let mut reader = BufReader::new(stdout).lines();
                    while let Ok(Some(line)) = reader.next_line().await {
                        if line.contains("sink") { let _ = tx_audio.send(Event::RefreshData).await; }
                    }
                }
            } else {
                // FALLBACK: Wenn pactl fehlt, pollen wir sehr schnell (250ms), 
                // damit die Hardware-Tasten verzögerungsfrei ins UI wandern.
                let mut interval = tokio::time::interval(std::time::Duration::from_millis(250));
                loop {
                    interval.tick().await;
                    let _ = tx_audio.send(Event::RefreshData).await;
                }
            }
        });

        // TASK 4: C++ Kommandos
        let tx_cmd = update_tx.clone();
        tokio::spawn(async move {
            let mut buf = vec![0u8; 1024]; 
            while let Ok(n) = rx.read(&mut buf).await {
                if n == 0 { break; } 
                if let Ok(cmd) = root_as_client_command(&buf[..n]) {
                    if let Some(action) = cmd.action() {
                        if action == "focus_workspace" {
                            let _ = Command::new("niri").args(&["msg", "action", "focus-workspace", &cmd.arg_int().to_string()]).output().await;
                        } else if action == "launch_menu" {
                            let _ = Command::new("niri").args(&["msg", "action", "spawn", "--", "fuzzel"]).output().await;
                        } else if action == "toggle_audio_mute" {
                            let _ = Command::new("wpctl").args(&["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"]).output().await;
                        }
                        // Zwingt die UI, sich nach einem Klick sofort neu zu zeichnen!
                        let _ = tx_cmd.send(Event::RefreshData).await; 
                    }
                }
            }
        });

        // TASK 5: Batterie & Fallback Timer
        let tx_timer = update_tx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                if tx_timer.send(Event::RefreshData).await.is_err() { break; }
            }
        });
       
        // --- NEUER TASK 6: Lauscht auf die Fernbedienung ---
        let tx_cli = update_tx.clone();
        let cli_listener_clone = cli_listener.clone(); // Wir erstellen den Klon...
        
        tokio::spawn(async move {
            let mut buf = [0u8; 32];
            loop {
                // ... und WICHTIG: Hier MUSS der Klon benutzt werden!
                if let Ok((len, _)) = cli_listener_clone.recv_from(&mut buf).await {
                    if &buf[..len] == b"TOGGLE_CC" {
                        if tx_cli.send(Event::ToggleControlCenter).await.is_err() {
                            break; 
                        }
                    }
                }
            }
        });

        let mut first_run = true;
        let mut cc_counter: u8 = 0; // Unser Zähler
        
        loop {
            if !first_run {
                match update_rx.recv().await {
                    Some(Event::ToggleControlCenter) => cc_counter = cc_counter.wrapping_add(1),
                    Some(Event::RefreshData) => {},
                    None => break,
                }
            }
            first_run = false;

            let (mut workspaces_data, active_title, (vol, muted)) = tokio::join!(
                fetch_workspaces(), fetch_active_window_title(), get_audio_state()
            );

            let mut builder = FlatBufferBuilder::new();

            // 1. Echter Workspace-Code
            workspaces_data.sort_by_key(|ws| ws.idx);
            let mut ws_offsets = Vec::new();
            for ws in workspaces_data {
                let name_str = ws.name.unwrap_or_else(|| ws.idx.to_string());
                let name_fb = builder.create_string(&name_str);
                ws_offsets.push(Workspace::create(&mut builder, &WorkspaceArgs {
                    id: ws.idx as _, name: Some(name_fb), is_active: ws.is_active,
                }));
            }
            let workspaces_vec = builder.create_vector(&ws_offsets);
            
            // 2. Fenster-Titel
            let title_fb = active_title.as_ref().map(|t| builder.create_string(t));
            
            // 3. Echtes Netzwerk (D-Bus)
            let net_name = get_network_name_dbus(&dbus_conn).await;
            let net_name_fb = builder.create_string(&net_name);

            let shell_state = ShellState::create(&mut builder, &ShellStateArgs {
                workspaces: Some(workspaces_vec),
                battery_percent: get_battery_percent(),
                active_window_title: title_fb,
                audio_volume: vol,
                audio_muted: muted,
                network_name: Some(net_name_fb),
                toggle_cc_signal: cc_counter, // HIER schicken wir das Signal!
            });

            builder.finish_size_prefixed(shell_state, None);
            if let Err(_) = tx.write_all(builder.finished_data()).await { break; }
        }
    }
}
