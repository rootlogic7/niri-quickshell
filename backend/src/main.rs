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
use std::process::Stdio;
use std::fs;
use serde::Deserialize;

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

// Sucht das aktive Fenster und gibt dessen Titel zurück
async fn fetch_active_window_title() -> Option<String> {
    let output = Command::new("niri").args(&["msg", "-j", "windows"]).output().await.ok()?;
    
    if let Ok(windows) = serde_json::from_slice::<Vec<NiriWindow>>(&output.stdout) {
        for w in windows {
            if w.is_focused {
                return w.title; // Gibt den Titel zurück, falls vorhanden
            }
        }
    }
    None
}

// Liest den Akku direkt aus dem Linux-Kernel aus (0 % CPU overhead)
fn get_battery_percent() -> i8 {
    if let Ok(bat) = fs::read_to_string("/sys/class/power_supply/BAT0/capacity") {
        return bat.trim().parse().unwrap_or(100);
    }
    if let Ok(bat) = fs::read_to_string("/sys/class/power_supply/BAT1/capacity") {
        return bat.trim().parse().unwrap_or(100);
    }
    100 // Fallback für Desktop-PCs
}

// OPTIMIERUNG 1: Keine Panics mehr! Gibt einfach eine leere Liste zurück, wenn Niri fehlt.
async fn fetch_workspaces() -> Vec<NiriWorkspace> {
    let output = match Command::new("niri").args(&["msg", "-j", "workspaces"]).output().await {
        Ok(o) => o,
        Err(e) => {
            eprintln!("⚠️ Konnte Niri nicht erreichen: {}", e);
            return vec![];
        }
    };

    let raw_json = String::from_utf8_lossy(&output.stdout);

    match serde_json::from_slice(&output.stdout) {
        Ok(workspaces) => workspaces,
        Err(e) => {
            eprintln!("❌ JSON Parse Fehler: {}", e);
            eprintln!("📦 Rohes Niri JSON: {}", raw_json);
            vec![]
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = "/tmp/niri-quickshell.sock";
    let _ = fs::remove_file(socket_path);

    let listener = UnixListener::bind(socket_path)?;
    println!("🚀 Rust Backend bereit! Warte auf Quickshell...");

    loop {
        // OPTIMIERUNG 2: Kein Absturz bei Verbindungsfehlern
        let stream = match listener.accept().await {
            Ok((s, _)) => s,
            Err(e) => {
                eprintln!("⚠️ Warnung: Fehler bei eingehender Socket-Verbindung: {}", e);
                continue; // Schleife läuft einfach weiter!
            }
        };
        
        println!("✅ Quickshell verbunden! Klinke in Niri-Events ein...");
        let (mut rx, mut tx) = tokio::io::split(stream);

        // TASK 1: Lauscht auf Niri-Events UND den Akku-Timer
        tokio::spawn(async move {
            let mut event_stream = match Command::new("niri")
                .args(&["msg", "-j", "event-stream"])
                .stdout(Stdio::piped())
                .spawn() 
            {
                Ok(child) => child,
                Err(e) => {
                    eprintln!("⚠️ Konnte niri event-stream nicht starten: {}", e);
                    return;
                }
            };

            if let Some(stdout) = event_stream.stdout.take() {
                let mut reader = BufReader::new(stdout).lines();
                // Timer, der jede Minute triggert
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));

                if !send_state_to_quickshell(&mut tx).await { return; }

                loop {
                    tokio::select! {
                        // Entweder Niri meldet einen Klick...
                        line = reader.next_line() => {
                            if let Ok(Some(_)) = line {
                                if !send_state_to_quickshell(&mut tx).await { break; }
                            } else {
                                break;
                            }
                        }
                        // ...oder eine Minute ist vergangen (Akku Update)
                        _ = interval.tick() => {
                            if !send_state_to_quickshell(&mut tx).await { break; }
                        }
                    }
                }
            }
        });

        // TASK 2: C++ Kommandos zu Niri
        tokio::spawn(async move {
            let mut buf = vec![0u8; 1024]; 
            
            while let Ok(n) = rx.read(&mut buf).await {
                if n == 0 { break; } 

                if let Ok(cmd) = root_as_client_command(&buf[..n]) {
                    if let Some(action) = cmd.action() {
                        if action == "focus_workspace" {
                            let ws_id = cmd.arg_int();
                            
                            // OPTIMIERUNG 3: output().await verhindert Zombie-Prozesse
                            let _ = Command::new("niri")
                                .args(&["msg", "action", "focus-workspace", &ws_id.to_string()])
                                .output()
                                .await;
                        }
                    }
                }
            }
        });
    }
}

async fn send_state_to_quickshell(tx: &mut tokio::io::WriteHalf<tokio::net::UnixStream>) -> bool {
    let mut workspaces_data = fetch_workspaces().await;
    workspaces_data.sort_by_key(|ws| ws.idx);
    let mut builder = FlatBufferBuilder::new();

    let mut ws_offsets = Vec::new();

    for ws in workspaces_data {
        let name_str = ws.name.unwrap_or_else(|| ws.idx.to_string());
        let name_fb = builder.create_string(&name_str);

        let ws_offset = Workspace::create(&mut builder, &WorkspaceArgs {
            id: ws.idx as _,
            name: Some(name_fb),
            is_active: ws.is_active,
        });
        ws_offsets.push(ws_offset);
    }

    let workspaces_vec = builder.create_vector(&ws_offsets);

    let active_title = fetch_active_window_title().await;
    let title_fb = active_title.as_ref().map(|t| builder.create_string(t));

    let shell_state = ShellState::create(&mut builder, &ShellStateArgs {
        workspaces: Some(workspaces_vec),
        battery_percent: get_battery_percent(),
        active_window_title: title_fb,
    });

    // NUR EINMAL ABSCHLIESSEN (mit Size Prefix!)
    builder.finish_size_prefixed(shell_state, None);
    let data = builder.finished_data();

    // Und ab damit in den Socket
    if let Err(e) = tx.write_all(data).await {
        eprintln!("❌ Verbindung zu Quickshell abgebrochen: {}", e);
        return false;
    }
    true
}
