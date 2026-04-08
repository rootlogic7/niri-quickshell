// src/modules/network.rs
use zbus::proxy;
use tokio::sync::mpsc;
use futures_lite::stream::StreamExt;
use crate::ipc::Event;

// --- D-BUS SCHNITTSTELLEN ---
// Diese Makros generieren automatisch Rust-Code, um mit dem NetworkManager zu sprechen
#[proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
trait NetworkManager {
    #[zbus(property)]
    fn primary_connection(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Connection.Active",
    default_service = "org.freedesktop.NetworkManager"
)]
trait ActiveConnection {
    #[zbus(property)]
    fn id(&self) -> zbus::Result<String>;
}

// --- DATENBESCHAFFUNG ---
// Fragt den echten Netzwerknamen über D-Bus ab
pub async fn get_network_name(conn: &zbus::Connection) -> String {
    // 1. Frage den NetworkManager nach der primären Verbindung
    if let Ok(nm_proxy) = NetworkManagerProxy::new(conn).await {
        if let Ok(path) = nm_proxy.primary_connection().await {
            // Wenn keine Verbindung besteht, ist der Pfad meistens "/"
            if path.as_str() != "/" {
                // 2. Erstelle einen Proxy für genau diese aktive Verbindung
                if let Ok(active_conn) = ActiveConnectionProxy::builder(conn)
                    .path(path)
                    .expect("Ungültiger D-Bus Pfad")
                    .build()
                    .await
                {
                    // 3. Hole den Namen (die "id") der Verbindung
                    if let Ok(id) = active_conn.id().await {
                        return id;
                    }
                }
            }
        }
    }
    
    // Fallback, wenn wir nichts finden konnten
    "Offline".to_string()
}

// --- EVENT LISTENER ---
// Lauscht auf Änderungen der primären Netzwerkverbindung
pub fn spawn_listener(tx: mpsc::Sender<Event>, conn: zbus::Connection) {
    tokio::spawn(async move {
        if let Ok(nm) = NetworkManagerProxy::new(&conn).await {
            // FIX: Die Methode gibt direkt den Stream zurück, kein Result!
            let mut stream = nm.receive_primary_connection_changed().await;
            
            while let Some(_) = stream.next().await {
                if tx.send(Event::RefreshData).await.is_err() {
                    break; // Channel zu, Task beenden
                }
            }
        } else {
            eprintln!("⚠️ Konnte NetworkManager D-Bus Listener nicht starten.");
        }
    });
}
