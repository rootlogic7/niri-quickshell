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
    id: u64,
    idx: u64,
    name: Option<String>,
    is_active: bool,
}

async fn fetch_workspaces() -> Vec<NiriWorkspace> {
    let output = Command::new("niri")
        .args(&["msg", "-j", "workspaces"])
        .output()
        .await
        .expect("Fehler beim Aufruf von niri msg");

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
        let (stream, _) = listener.accept().await?;
        println!("✅ Quickshell verbunden! Klinke in Niri-Events ein...");

        // Wir spalten den Socket auf in Lesen (rx) und Schreiben (tx)
        let (mut rx, mut tx) = tokio::io::split(stream);

        // TASK 1: Lauscht auf Niri-Events und schreibt an C++ (Nutzt `tx`)
        tokio::spawn(async move {
            let mut event_stream = Command::new("niri")
                .args(&["msg", "-j", "event-stream"])
                .stdout(Stdio::piped())
                .spawn()
                .expect("Konnte niri event-stream nicht starten");

            let stdout = event_stream.stdout.take().unwrap();
            let mut reader = BufReader::new(stdout).lines();

            // Erster Push
            if !send_state_to_quickshell(&mut tx).await {
                return;
            }

            // Schleife
            while let Ok(Some(_line)) = reader.next_line().await {
                if !send_state_to_quickshell(&mut tx).await {
                    break; 
                }
            }
        });

        // TASK 2: Lauscht auf Befehle von C++ und steuert Niri (Nutzt `rx`)
        tokio::spawn(async move {
            let mut buf = vec![0u8; 1024]; 
            
            while let Ok(n) = rx.read(&mut buf).await {
                if n == 0 { break; } 

                if let Ok(cmd) = root_as_client_command(&buf[..n]) {
                    if let Some(action) = cmd.action() {
                        if action == "focus_workspace" {
                            let ws_id = cmd.arg_int();
                            println!("🎯 UI Befehl empfangen: Wechsle zu Workspace {}", ws_id);
                            
                            let _ = Command::new("niri")
                                .args(&["msg", "action", "focus-workspace", &ws_id.to_string()])
                                .spawn();
                        }
                    }
                }
            }
        });
    }
}

// Wir übergeben hier 'tx' statt 'stream'
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

    let shell_state = ShellState::create(&mut builder, &ShellStateArgs {
        workspaces: Some(workspaces_vec),
    });

    builder.finish(shell_state, None);
    let data = builder.finished_data();

    // HIER NUTZEN WIR NUN tx statt stream
    if let Err(e) = tx.write_all(data).await {
        eprintln!("❌ Verbindung zu Quickshell abgebrochen: {}", e);
        return false;
    }
    true
}
