// src/modules/battery.rs
use std::fs;
use tokio::sync::mpsc;
use crate::ipc::Event;

// Holt den aktuellen Akkustand
pub fn get_battery_percent() -> i8 { 
    fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
        .unwrap_or_default()
        .trim()
        .parse()
        .unwrap_or(100) 
}

// Startet den Timer für regelmäßige Updates
pub fn spawn_listener(tx: mpsc::Sender<Event>) {
    tokio::spawn(async move {
        // Alle 60 Sekunden aktualisieren
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            if tx.send(Event::RefreshData).await.is_err() { 
                break; // Channel geschlossen, Task beenden
            }
        }
    });
}
