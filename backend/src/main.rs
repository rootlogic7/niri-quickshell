mod shell_state_generated;

use shell_state_generated::niri_shell::{
    ShellState, ShellStateArgs, Workspace, WorkspaceArgs,
};
use flatbuffers::FlatBufferBuilder;
use tokio::net::UnixListener;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
use tokio::process::Command;
use std::process::Stdio;
use std::fs;
use serde::Deserialize;

// 1. Das Datenmodell: Wie Niri uns die Workspaces als JSON liefert
#[derive(Deserialize, Debug)]
struct NiriWorkspace {
    id: u64,
    name: Option<String>,
    is_active: bool,
}

// 2. Die Aggregator-Funktion: Holt den absoluten, fehlerfreien Niri-Zustand
async fn fetch_workspaces() -> Vec<NiriWorkspace> {
    let output = Command::new("niri")
        .args(&["msg", "-j", "workspaces"])
        .output()
        .await
        .expect("Fehler beim Aufruf von niri msg");

    // Wir wandeln die rohen Bytes kurz in einen String um, damit wir ihn lesen können
    let raw_json = String::from_utf8_lossy(&output.stdout);

    match serde_json::from_slice(&output.stdout) {
        Ok(workspaces) => workspaces,
        Err(e) => {
            // Wenn das Parsen fehlschlägt, schreien wir den Fehler laut in die Konsole!
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
        let (mut stream, _) = listener.accept().await?;
        println!("✅ Quickshell verbunden! Klinke in Niri-Events ein...");

        tokio::spawn(async move {
            // Wir starten den Niri-Event-Stream im Hintergrund (Lauscht auf jede Änderung)
            let mut event_stream = Command::new("niri")
                .args(&["msg", "-j", "event-stream"])
                .stdout(Stdio::piped())
                .spawn()
                .expect("Konnte niri event-stream nicht starten");

            let stdout = event_stream.stdout.take().unwrap();
            let mut reader = BufReader::new(stdout).lines();

            // Erster initialer Push, damit das UI beim Start sofort Daten hat
            if !send_state_to_quickshell(&mut stream).await {
                return;
            }

            // Endlosschleife: Wir schlafen bei 0% CPU, bis Niri uns ein Event schickt
            while let Ok(Some(_line)) = reader.next_line().await {
                // Ein Event! Niri hat sich verändert. Wir holen den neuen Zustand und senden ihn ab.
                if !send_state_to_quickshell(&mut stream).await {
                    break; // Frontend hat sich getrennt
                }
            }
        });
    }
}

// 3. Die Zero-Copy-Brücke: Baut den FlatBuffer und schickt ihn via Socket ab
async fn send_state_to_quickshell(stream: &mut tokio::net::UnixStream) -> bool {
    let workspaces_data = fetch_workspaces().await;
    let mut builder = FlatBufferBuilder::new();

    let mut ws_offsets = Vec::new();

    for ws in workspaces_data {
        // Fallback: Wenn Niri keinen expliziten Namen hat, nehmen wir die ID als String
        let name_str = ws.name.unwrap_or_else(|| ws.id.to_string());
        let name_fb = builder.create_string(&name_str);

        // Wir nutzen den generierten Code aus unserem .fbs Vertrag
        let ws_offset = Workspace::create(&mut builder, &WorkspaceArgs {
            id: ws.id,
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

    // Den puren Speicherblock direkt in C++ feuern
    if let Err(e) = stream.write_all(data).await {
        eprintln!("❌ Verbindung zu Quickshell abgebrochen: {}", e);
        return false;
    }
    true
}
