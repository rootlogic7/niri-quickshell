use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};
use std::thread;
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Config};
use std::sync::mpsc::channel;

#[derive(Deserialize, Clone)]
pub struct ThemeConfig {
    pub bg_color: String,
    pub fg_color: String,
    pub accent_color: String,
}

// Unser Fallback, falls die Datei noch nicht existiert (Catppuccin Macchiato)
impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            bg_color: "#24273a".to_string(), 
            fg_color: "#cad3f5".to_string(),
            accent_color: "#8aadf4".to_string(),
        }
    }
}

// Globale, thread-sichere Variable für das aktuelle Theme
static CURRENT_THEME: OnceLock<RwLock<ThemeConfig>> = OnceLock::new();

fn theme_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
    path.push("niri-quickshell/theme.toml");
    path
}

pub fn get_theme() -> ThemeConfig {
    CURRENT_THEME
        .get_or_init(|| RwLock::new(ThemeConfig::default()))
        .read()
        .unwrap()
        .clone()
}

pub fn init_watcher() {
    // Variable initialisieren
    let _ = CURRENT_THEME.get_or_init(|| RwLock::new(ThemeConfig::default()));

    // 1. Initiales Laden beim Start
    load_theme_from_disk();

    // 2. Watcher in einem Hintergrund-Thread starten
    thread::spawn(|| {
        let (tx, rx) = channel();
        let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();
        
        let path = theme_path();
        // Ordner erstellen, falls er nicht existiert
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        
        // Wächter an die Datei oder den Ordner heften
        let _ = watcher.watch(&path.parent().unwrap(), RecursiveMode::NonRecursive);

        // Auf Speichern-Events lauschen
        for res in rx {
            if let Ok(event) = res {
                // Wenn sich eine Datei im Ordner ändert und es unsere theme.toml ist:
                if event.kind.is_modify() && event.paths.iter().any(|p| p.ends_with("theme.toml")) {
                    println!("🎨 Theme-Datei gespeichert! Lade neue Farben...");
                    load_theme_from_disk();
                }
            }
        }
    });
}

fn load_theme_from_disk() {
    let path = theme_path();
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(new_theme) = toml::from_str::<ThemeConfig>(&content) {
            if let Some(lock) = CURRENT_THEME.get() {
                if let Ok(mut theme) = lock.write() {
                    *theme = new_theme;
                }
            }
        } else {
            println!("⚠️ Fehler beim Parsen der theme.toml. Syntax prüfen!");
        }
    }
}
