mod shell_state_generated;

use shell_state_generated::niri_shell::{
    ShellState, ShellStateArgs, Workspace, WorkspaceArgs,
};
use flatbuffers::FlatBufferBuilder;
use tokio::net::UnixListener;
use tokio::io::AsyncWriteExt;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = "/tmp/niri-quickshell.sock";

    // Alten Socket aufräumen, falls der Daemon gecrasht ist
    let _ = fs::remove_file(socket_path);

    // Unseren eigenen Server-Socket für Quickshell starten
    let listener = UnixListener::bind(socket_path)?;
    println!("🚀 Rust Backend läuft! Warte auf Quickshell an {}...", socket_path);

    // Endlosschleife, die auf Verbindungen von Quickshell wartet
    loop {
        let (mut stream, _) = listener.accept().await?;
        println!("✅ Quickshell (C++ Plugin) hat sich verbunden!");

        // Hier würden wir normalerweise asynchron Niri-Events via NIRI_SOCKET lesen.
        // Für diesen MVP generieren wir einmalig einen FlatBuffer mit Demo-Daten:
        
        let mut builder = FlatBufferBuilder::new();

        // 1. Strings im FlatBuffer erstellen
        let ws1_name = builder.create_string("1: Browser");
        let ws2_name = builder.create_string("2: Code");

        // 2. Workspaces erstellen
        let ws1 = Workspace::create(&mut builder, &WorkspaceArgs {
            id: 1,
            name: Some(ws1_name),
            is_active: true,
        });

        let ws2 = Workspace::create(&mut builder, &WorkspaceArgs {
            id: 2,
            name: Some(ws2_name),
            is_active: false,
        });

        // 3. Liste (Vector) der Workspaces erstellen
        let workspaces_vec = builder.create_vector(&[ws1, ws2]);

        // 4. Das finale ShellState-Objekt zusammenbauen
        let shell_state = ShellState::create(&mut builder, &ShellStateArgs {
            workspaces: Some(workspaces_vec),
        });

        // 5. Den Buffer abschließen
        builder.finish(shell_state, None);

        // 6. Die fertigen, puren Bytes holen...
        let data = builder.finished_data();

        // ... und ohne Overhead direkt in den Socket zu Quickshell schieben!
        if let Err(e) = stream.write_all(data).await {
            eprintln!("Fehler beim Senden an Quickshell: {}", e);
        } else {
            println!("📦 Zero-Copy FlatBuffer ({} Bytes) erfolgreich gesendet!", data.len());
        }
    }
}
