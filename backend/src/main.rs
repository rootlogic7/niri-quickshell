mod shell_state_generated;
mod client_command_generated;
mod ipc;
mod modules;
mod state_store;

use tokio::net::{UnixListener, UnixDatagram};
use tokio::sync::mpsc;
use std::fs;
use std::sync::Arc;
use crate::ipc::{Event, get_socket_path};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    crate::modules::theme::init_watcher();
    let args: Vec<String> = std::env::args().collect();
    
    // Holen uns jetzt die sicheren ~/.config oder /run/user Pfade!
    let cli_socket_path = get_socket_path("cli.sock");
    let daemon_socket_path = get_socket_path("ipc.sock");

    // --- CLI MODUS ---
    if args.len() > 1 && args[1] == "toggle-cc" {
        let socket = UnixDatagram::unbound()?;
        // Hier den PathBuf in einen String umwandeln (oder als C-String)
        let _ = socket.send_to(b"TOGGLE_CC", &cli_socket_path);
        return Ok(());
    }

    // --- DAEMON MODUS ---
    let _ = fs::remove_file(&daemon_socket_path);
    let _ = fs::remove_file(&cli_socket_path);
    
    let listener = UnixListener::bind(&daemon_socket_path)?;
    let cli_listener = Arc::new(UnixDatagram::bind(&cli_socket_path)?);
    let dbus_conn = zbus::Connection::system().await?;
    
    println!("🚀 Niri-Quickshell Backend lauscht auf {:?}", daemon_socket_path);

    loop {
        // Warten auf das C++ Frontend
        let stream = match listener.accept().await { Ok((s, _)) => s, Err(_) => continue };
        let (rx, mut tx) = tokio::io::split(stream);
        
        let (update_tx, mut update_rx) = mpsc::channel::<Event>(20);

        // 1. Alle Module starten (Die werfen Events in den update_tx Channel)
        modules::niri::spawn_listener(update_tx.clone());
        modules::audio::spawn_listener(update_tx.clone());
        modules::network::spawn_listener(update_tx.clone(), dbus_conn.clone());
        modules::battery::spawn_listener(update_tx.clone());
        
        // 2. Netzwerk-Listener starten (Empfangen C++ Klicks und CLI-Befehle)
        ipc::spawn_client_command_listener(rx, update_tx.clone());
        ipc::spawn_cli_listener(cli_listener.clone(), update_tx.clone());

        // 3. Die Hauptschleife (Data-Refresh & Versand)
        let mut first_run = true;
        let mut cc_counter: u8 = 0;
        
        loop {
            // Beim ersten Lauf sofort senden, danach auf Events warten
            if !first_run {
                match update_rx.recv().await {
                    Some(Event::ToggleControlCenter) => cc_counter = cc_counter.wrapping_add(1),
                    Some(Event::RefreshData) => {}, // Einfach refreshen
                    None => break, // Channel geschlossen (Fehler)
                }
            }
            first_run = false;

            // Delegiert an den State-Store
            if state_store::build_and_send(&mut tx, &dbus_conn, cc_counter).await.is_err() {
                eprintln!("⚠️ Verbindung zum C++ Frontend getrennt. Warte auf Reconnect...");
                break; // Bricht die innere Schleife ab, äußere loop wartet auf neuen connect
            }
        }
    }
}
