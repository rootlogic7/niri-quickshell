mod shell_state_generated;
mod client_command_generated;

use shell_state_generated::niri_shell::{
    ShellState, ShellStateArgs, Workspace, WorkspaceArgs,
};
use client_command_generated::niri_shell::root_as_client_command;
use flatbuffers::FlatBufferBuilder;
use tokio::net::UnixListener;
use tokio::io::{AsyncWriteExt, AsyncReadExt, AsyncBufReadExt, BufReader};
use tokio::process::Command; // <-- Ab sofort ist alles non-blocking!
use tokio::sync::mpsc;
use std::process::Stdio;
use std::fs;
use serde::Deserialize;
use futures_lite::stream::StreamExt;

// --- D-BUS SCHNITTSTELLEN ---
#[zbus::proxy(interface = "org.freedesktop.NetworkManager", default_service = "org.freedesktop.NetworkManager", default_path = "/org/freedesktop/NetworkManager")]
trait NetworkManager {
    #[zbus(property)]
    fn primary_connection(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
}

#[zbus::proxy(interface = "org.freedesktop.NetworkManager.Connection.Active", default_service = "org.freedesktop.NetworkManager")]
trait ActiveConnection {
    #[zbus(property)]
    fn id(&self) -> zbus::Result<String>;
}

// --- STRUKTUREN ---
#[derive(Deserialize, Debug)]
struct NiriWorkspace {
    #[allow(dead_code)]
    id: u64,
    idx: u64,
    name: Option<String>,
    is_active: bool,
}

#[derive(Deserialize, Debug)]
struct NiriWindow {
    title: Option<String>,
    is_focused: bool,
}

// --- ASYNCHRONE HILFSFUNKTIONEN ---
async fn fetch_workspaces() -> Vec<NiriWorkspace> {
    let output = match Command::new("niri").args(&["msg", "-j", "workspaces"]).output().await {
        Ok(o) => o,
        Err(_) => return vec![],
    };
    serde_json::from_slice(&output.stdout).unwrap_or_default()
}

async fn fetch_active_window_title() -> Option<String> {
    let output = Command::new("niri").args(&["msg", "-j", "windows"]).output().await.ok()?;
    if let Ok(windows) = serde_json::from_slice::<Vec<NiriWindow>>(&output.stdout) {
        for w in windows {
            if w.is_focused { return w.title; }
        }
    }
    None
}

// NEU: Jetzt 100% async! Blockiert den Thread nicht mehr.
async fn get_audio_state() -> (i8, bool) {
    if let Ok(output) = Command::new("wpctl").args(&["get-volume", "@DEFAULT_AUDIO_SINK@"]).output().await {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut volume: i8 = 0;
        let muted = stdout.contains("[MUTED]");
        if let Some(vol_str) = stdout.split_whitespace().nth(1) {
            if let Ok(vol_float) = vol_str.parse::<f32>() {
                volume = (vol_float * 100.0).round() as i8;
            }
        }
        return (volume, muted);
    }
    (0, true)
}

fn get_battery_percent() -> i8 {
    if let Ok(bat) = fs::read_to_string("/sys/class/power_supply/BAT0/capacity") { return bat.trim().parse().unwrap_or(100); }
    if let Ok(bat) = fs::read_to_string("/sys/class/power_supply/BAT1/capacity") { return bat.trim().parse().unwrap_or(100); }
    100 
}

async fn get_network_name_dbus(conn: &zbus::Connection) -> String {
    if let Ok(nm) = NetworkManagerProxy::new(conn).await {
        if let Ok(path) = nm.primary_connection().await {
            if path.as_str() != "/" {
                if let Ok(active) = ActiveConnectionProxy::builder(conn).path(path).unwrap().build().await {
                    if let Ok(id) = active.id().await { return id; }
                }
            }
        }
    }
    "Offline".to_string()
}

// --- HAUPTSCHLEIFE ---
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = "/tmp/niri-quickshell.sock";
    let _ = fs::remove_file(socket_path);
    let listener = UnixListener::bind(socket_path)?;
    let dbus_conn = zbus::Connection::system().await?;
    
    println!("🚀 Rust Backend bereit! Warte auf Quickshell...");

    loop {
        let stream = match listener.accept().await {
            Ok((s, _)) => s,
            Err(_) => continue,
        };
        
        let (mut rx, mut tx) = tokio::io::split(stream);
        let (update_tx, mut update_rx) = mpsc::channel::<()>(20);

        // TASK 1: Niri Event Listener
        let tx_niri = update_tx.clone();
        tokio::spawn(async move {
            if let Ok(mut event_stream) = Command::new("niri").args(&["msg", "-j", "event-stream"]).stdout(Stdio::piped()).spawn() {
                if let Some(stdout) = event_stream.stdout.take() {
                    let mut reader = BufReader::new(stdout).lines();
                    while let Ok(Some(_)) = reader.next_line().await { let _ = tx_niri.send(()).await; }
                }
            }
        });

        // TASK 2: NetworkManager D-Bus Listener
        let tx_nm = update_tx.clone();
        let dbus_conn_clone = dbus_conn.clone();
        tokio::spawn(async move {
            if let Ok(nm) = NetworkManagerProxy::new(&dbus_conn_clone).await {
                let mut stream = nm.receive_primary_connection_changed().await;
                while let Some(_) = stream.next().await { let _ = tx_nm.send(()).await; }
            }
        });

        // TASK 3: Audio Event Listener mit NixOS-Fallback
        let tx_audio = update_tx.clone();
        tokio::spawn(async move {
            if let Ok(mut child) = Command::new("pactl").args(&["subscribe"]).stdout(Stdio::piped()).spawn() {
                if let Some(stdout) = child.stdout.take() {
                    let mut reader = BufReader::new(stdout).lines();
                    while let Ok(Some(line)) = reader.next_line().await {
                        if line.contains("sink") { let _ = tx_audio.send(()).await; }
                    }
                }
            } else {
                // FALLBACK: Wenn pactl fehlt, pollen wir sehr schnell (250ms), 
                // damit die Hardware-Tasten verzögerungsfrei ins UI wandern.
                let mut interval = tokio::time::interval(std::time::Duration::from_millis(250));
                loop {
                    interval.tick().await;
                    let _ = tx_audio.send(()).await;
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
                        let _ = tx_cmd.send(()).await; 
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
                if tx_timer.send(()).await.is_err() { break; }
            }
        });

        // --- HAUPT EVENT LOOP ---
        let mut first_run = true;
        
        loop {
            if !first_run {
                if update_rx.recv().await.is_none() { break; }
            }
            first_run = false;

            // NEU: HIER IST DIE MAGIE! tokio::join! führt alle 3 Abfragen PARALLEL aus!
            let (mut workspaces_data, active_title, (vol, muted)) = tokio::join!(
                fetch_workspaces(),
                fetch_active_window_title(),
                get_audio_state()
            );

            workspaces_data.sort_by_key(|ws| ws.idx);
            let mut builder = FlatBufferBuilder::new();

            let mut ws_offsets = Vec::new();
            for ws in workspaces_data {
                let name_str = ws.name.unwrap_or_else(|| ws.idx.to_string());
                let name_fb = builder.create_string(&name_str);
                ws_offsets.push(Workspace::create(&mut builder, &WorkspaceArgs {
                    id: ws.idx as _, name: Some(name_fb), is_active: ws.is_active,
                }));
            }
            let workspaces_vec = builder.create_vector(&ws_offsets);
            
            let title_fb = active_title.as_ref().map(|t| builder.create_string(t));
            let net_name = get_network_name_dbus(&dbus_conn).await;
            let net_name_fb = builder.create_string(&net_name);

            let shell_state = ShellState::create(&mut builder, &ShellStateArgs {
                workspaces: Some(workspaces_vec),
                battery_percent: get_battery_percent(),
                active_window_title: title_fb,
                audio_volume: vol,
                audio_muted: muted,
                network_name: Some(net_name_fb),
            });

            builder.finish_size_prefixed(shell_state, None);
            if let Err(_) = tx.write_all(builder.finished_data()).await {
                break;
            }
        }
    }
}
