use std::env;
use std::path::PathBuf;
use std::fs;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, ReadHalf};
use tokio::net::{UnixStream, UnixDatagram};
use tokio::sync::mpsc;
use tokio::process::Command;
use crate::client_command_generated::niri_shell::root_as_client_command;

// Unsere zentralen Events
pub enum Event {
    RefreshData,
    ToggleControlCenter,
}

// Sichere, XDG-konforme Socket-Pfade generieren
pub fn get_socket_path(filename: &str) -> PathBuf {
    let runtime_dir = env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| format!("/run/user/{}", unsafe { libc::getuid() }));
    
    let app_dir = PathBuf::from(runtime_dir).join("niri-quickshell");
    
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir).expect("Konnte XDG Runtime Dir nicht erstellen");
    }
    
    app_dir.join(filename)
}

// Lauscht auf Befehle vom C++ Frontend (z.B. Klicks)
pub fn spawn_client_command_listener(mut rx: ReadHalf<UnixStream>, tx: mpsc::Sender<Event>) {
    tokio::spawn(async move {
        let mut buf = vec![0u8; 1024]; 
        while let Ok(n) = rx.read(&mut buf).await {
            if n == 0 { break; } // Verbindung getrennt
            
            if let Ok(cmd) = root_as_client_command(&buf[..n]) {
                if let Some(action) = cmd.action() {
                    // TODO: Das Ausführen der Commands kann später in die jeweiligen Module wandern
                    if action == "focus_workspace" {
                        let _ = Command::new("niri").args(&["msg", "action", "focus-workspace", &cmd.arg_int().to_string()]).output().await;
                    } else if action == "launch_menu" {
                        let _ = Command::new("niri").args(&["msg", "action", "spawn", "--", "fuzzel"]).output().await;
                    } else if action == "toggle_audio_mute" {
                        let _ = Command::new("wpctl").args(&["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"]).output().await;
                    }
                    
                    // Zwingt die UI, sich nach einem Klick sofort neu zu zeichnen
                    let _ = tx.send(Event::RefreshData).await; 
                }
            }
        }
    });
}

// Lauscht auf die CLI-Fernbedienung (z.B. von Hotkeys)
pub fn spawn_cli_listener(cli_listener: Arc<UnixDatagram>, tx: mpsc::Sender<Event>) {
    tokio::spawn(async move {
        let mut buf = [0u8; 32];
        loop {
            if let Ok((len, _)) = cli_listener.recv_from(&mut buf).await {
                if &buf[..len] == b"TOGGLE_CC" {
                    if tx.send(Event::ToggleControlCenter).await.is_err() {
                        break; 
                    }
                }
            }
        }
    });
}
