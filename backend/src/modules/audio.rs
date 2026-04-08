// src/modules/audio.rs
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use std::process::Stdio;
use tokio::sync::mpsc;
use crate::ipc::Event;

// Holt den aktuellen Audio-Status (Lautstärke 0-100, Muted) via wpctl (WirePlumber/PipeWire)
pub async fn get_audio_state() -> (i8, bool) {
    if let Ok(o) = Command::new("wpctl").args(&["get-volume", "@DEFAULT_AUDIO_SINK@"]).output().await {
        let s = String::from_utf8_lossy(&o.stdout);
        let vol = s.split_whitespace()
            .nth(1)
            .and_then(|v| v.parse::<f32>().ok())
            .map(|v| (v * 100.0) as i8)
            .unwrap_or(0);
            
        return (vol, s.contains("[MUTED]"));
    }
    
    // Fallback: 0% Lautstärke, stummgeschaltet, falls wpctl fehlschlägt oder nicht installiert ist
    (0, true) 
}

// Startet den Listener für Audio-Events
pub fn spawn_listener(tx: mpsc::Sender<Event>) {
    tokio::spawn(async move {
        // Versuche pactl (PulseAudio/PipeWire-Pulse Kompatibilitätsschicht) für ressourcenschonende Events
        if let Ok(mut child) = Command::new("pactl").args(&["subscribe"]).stdout(Stdio::piped()).spawn() {
            if let Some(stdout) = child.stdout.take() {
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    // Wir aktualisieren die UI nur, wenn sich am Audio-Ausgang ("sink") etwas ändert
                    if line.contains("sink") { 
                        let _ = tx.send(Event::RefreshData).await; 
                    }
                }
            }
        } else {
            // FALLBACK: Wenn pactl fehlt (z.B. auf einem System ohne PulseAudio-Kompatibilität), 
            // pollen wir in sehr kurzen Abständen, damit die Hardware-Tasten verzögerungsfrei ins UI wandern.
            // (In Zukunft könnten wir hier z.B. einen nativen WirePlumber D-Bus Listener einbauen)
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(250));
            loop {
                interval.tick().await;
                if tx.send(Event::RefreshData).await.is_err() {
                    break; // Channel geschlossen
                }
            }
        }
    });
}
