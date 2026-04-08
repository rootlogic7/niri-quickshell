// src/modules/niri.rs
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use std::process::Stdio;
use tokio::sync::mpsc;
use serde::Deserialize;
use crate::ipc::Event;

// --- JSON Datenstrukturen ---
#[derive(Deserialize, Debug)] 
pub struct NiriWorkspace { 
    #[allow(dead_code)]
    pub id: u64, 
    pub idx: u64, 
    pub name: Option<String>, 
    pub is_active: bool 
}

#[derive(Deserialize, Debug)] 
pub struct NiriWindow { 
    pub title: Option<String>, 
    pub is_focused: bool 
}

// --- Datenbeschaffung ---

// Holt alle aktuellen Workspaces
pub async fn fetch_workspaces() -> Vec<NiriWorkspace> {
    if let Ok(output) = Command::new("niri").args(&["msg", "-j", "workspaces"]).output().await {
        serde_json::from_slice(&output.stdout).unwrap_or_default()
    } else {
        vec![]
    }
}

// Holt den Titel des aktuell fokussierten Fensters
pub async fn fetch_active_window_title() -> Option<String> {
    if let Ok(output) = Command::new("niri").args(&["msg", "-j", "windows"]).output().await {
        if let Ok(windows) = serde_json::from_slice::<Vec<NiriWindow>>(&output.stdout) {
            return windows.into_iter()
                .find(|w| w.is_focused)
                .and_then(|w| w.title);
        }
    }
    None
}

// --- Event Listener ---

// Lauscht auf den Niri-Event-Stream und feuert bei jeder Änderung ein Event
pub fn spawn_listener(tx: mpsc::Sender<Event>) {
    tokio::spawn(async move {
        if let Ok(mut event_stream) = Command::new("niri")
            .args(&["msg", "-j", "event-stream"])
            .stdout(Stdio::piped())
            .spawn() 
        {
            if let Some(stdout) = event_stream.stdout.take() {
                let mut reader = BufReader::new(stdout).lines();
                // Niri spuckt für jedes Event eine neue JSON-Zeile aus.
                // Uns reicht aktuell die Info *dass* etwas passiert ist, um einen kompletten State-Refresh anzustoßen.
                while let Ok(Some(_)) = reader.next_line().await { 
                    if tx.send(Event::RefreshData).await.is_err() {
                        break; // Channel zu, wir beenden den Task
                    }
                }
            }
        } else {
            eprintln!("⚠️ Konnte Niri Event-Stream nicht starten. Läuft Niri?");
        }
    });
}
