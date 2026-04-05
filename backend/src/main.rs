mod shell_state_generated;
mod client_command_generated;

use shell_state_generated::niri_shell::{
    ShellState, ShellStateArgs, Workspace, WorkspaceArgs,
};
use client_command_generated::niri_shell::root_as_client_command;
use flatbuffers::FlatBufferBuilder;
use tokio::net::UnixListener;
use tokio::io::{AsyncWriteExt, AsyncReadExt, AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc; // <--- NEU: Unser Event-Channel
use std::process::Stdio;
use std::fs;
use serde::Deserialize;
use futures_lite::stream::StreamExt;

// --- D-BUS SCHNITTSTELLEN FÜR NETWORKMANAGER ---

#[zbus::proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
trait NetworkManager {
    // D-Bus Property für die aktuell aktive Verbindung
    #[zbus(property)]
    fn primary_connection(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
}

#[zbus::proxy(
    interface = "org.freedesktop.NetworkManager.Connection.Active",
    default_service = "org.freedesktop.NetworkManager"
)]
trait ActiveConnection {
    // D-Bus Property für den Namen (SSID) der Verbindung
    #[zbus(property)]
    fn id(&self) -> zbus::Result<String>;
}

// ------------------------------------------------

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

// --- HILFSFUNKTIONEN ---

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

fn get_battery_percent() -> i8 {
    if let Ok(bat) = fs::read_to_string("/sys/class/power_supply/BAT0/capacity") {
        return bat.trim().parse().unwrap_or(100);
    }
    if let Ok(bat) = fs::read_to_string("/sys/class/power_supply/BAT1/capacity") {
        return bat.trim().parse().unwrap_or(100);
    }
    100 
}

fn get_audio_state() -> (i8, bool) {
    if let Ok(output) = std::process::Command::new("wpctl").args(&["get-volume", "@DEFAULT_AUDIO_SINK@"]).output() {
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

// Liest asynchron (ohne Prozess-Spawning) direkt aus dem D-Bus!
async fn get_network_name_dbus(conn: &zbus::Connection) -> String {
    if let Ok(nm) = NetworkManagerProxy::new(conn).await {
        if let Ok(path) = nm.primary_connection().await {
            if path.as_str() != "/" {
                // Verbinde zum spezifischen Pfad der aktiven Verbindung
                if let Ok(active) = ActiveConnectionProxy::builder(conn).path(path).unwrap().build().await {
                    if let Ok(id) = active.id().await {
                        return id;
                    }
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
    
    // Baue EINMALIGE Verbindung zum System-D-Bus auf
    let dbus_conn = zbus::Connection::system().await?;
    
    println!("🚀 Rust Backend bereit! Warte auf Quickshell...");

    loop {
        let stream = match listener.accept().await {
            Ok((s, _)) => s,
            Err(_) => continue,
        };
        
        println!("✅ Quickshell verbunden! Initialisiere Event-Kanäle...");
        let (mut rx, mut tx) = tokio::io::split(stream);

        // Wir erstellen einen Channel. Wenn IRGENDWER (Niri, NetworkManager, Timer)
        // () in diesen Sender (update_tx) wirft, pusht die Hauptschleife an Quickshell!
        let (update_tx, mut update_rx) = mpsc::channel::<()>(20);

        // --- TASK 1: Niri Event Listener ---
        let tx_niri = update_tx.clone();
        tokio::spawn(async move {
            if let Ok(mut event_stream) = Command::new("niri").args(&["msg", "-j", "event-stream"]).stdout(Stdio::piped()).spawn() {
                if let Some(stdout) = event_stream.stdout.take() {
                    let mut reader = BufReader::new(stdout).lines();
                    while let Ok(Some(_)) = reader.next_line().await {
                        let _ = tx_niri.send(()).await; // Melde: Niri hat sich geändert!
                    }
                }
            }
        });

        // --- TASK 2: NetworkManager D-Bus Listener ---
        let tx_nm = update_tx.clone();
        let dbus_conn_clone = dbus_conn.clone();
        tokio::spawn(async move {
            if let Ok(nm) = NetworkManagerProxy::new(&dbus_conn_clone).await {
                // HIER IST DER FIX: Die Funktion gibt den Stream direkt zurück, kein Result!
                let mut stream = nm.receive_primary_connection_changed().await;
                
                while let Some(_) = stream.next().await {
                    let _ = tx_nm.send(()).await; // Melde: WLAN hat sich geändert!
                }
            }
        });

        // --- TASK 3: Audio & Sensor Timer (Polling) ---
        // Audio könnte man auch per D-Bus/Pipewire machen, aber als Einstieg belassen 
        // wir hier den einfachen Timer.
        let tx_timer = update_tx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
            loop {
                interval.tick().await;
                if tx_timer.send(()).await.is_err() { break; }
            }
        });

        // --- TASK 4: C++ Kommandos zu Niri / Audio ---
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
                    }
                }
            }
        });

        // --- HAUPT EVENT LOOP ---
        // Dies ist der "Dirigent". Er wartet auf ein Signal aus einem der obigen Tasks.
        // Sobald eins kommt, schnürt er das Paket.
        let mut first_run = true;
        
        loop {
            if !first_run {
                if update_rx.recv().await.is_none() { break; } // Warte auf irgendein Update!
            }
            first_run = false;

            let mut workspaces_data = fetch_workspaces().await;
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
            
            let active_title = fetch_active_window_title().await;
            let title_fb = active_title.as_ref().map(|t| builder.create_string(t));

            let (vol, muted) = get_audio_state();
            
            // D-BUS Abfrage statt nmcli (0% CPU, 0 Spawns)
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
                break; // Socket tot -> Schleife beenden, auf Reconnect warten
            }
        }
    }
}
