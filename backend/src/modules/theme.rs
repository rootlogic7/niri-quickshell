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

    pub color0: Option<String>,
    pub color1: Option<String>,
    pub color2: Option<String>,
    pub color3: Option<String>,
    pub color4: Option<String>,
    pub color5: Option<String>,
    pub color6: Option<String>,
    pub color7: Option<String>,
    pub color8: Option<String>,
    pub color9: Option<String>,
    pub color10: Option<String>,
    pub color11: Option<String>,
    pub color12: Option<String>,
    pub color13: Option<String>,
    pub color14: Option<String>,
    pub color15: Option<String>,
}

// Unser Fallback, falls die Datei noch nicht existiert (Catppuccin Macchiato)
impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            bg_color: "#24273a".to_string(), 
            fg_color: "#cad3f5".to_string(),
            accent_color: "#8aadf4".to_string(),

            color0: None, color1: None, color2: None, color3: None,
            color4: None, color5: None, color6: None, color7: None,
            color8: None, color9: None, color10: None, color11: None,
            color12: None, color13: None, color14: None, color15: None,
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
                    *theme = new_theme.clone();
                }
            }
            
            // NEU: Die Exporter sofort nach dem Laden triggern!
            crate::modules::exporter::export_fuzzel(&new_theme);
            crate::modules::exporter::export_ghostty(&new_theme);
            // NEU: Ghostty per Signal zwingen, die externe Datei sofort neu zu laden!
            let _ = std::process::Command::new("pkill")
                .args(&["-USR2", "ghostty"])
                .spawn();
                
            // NEU: Den Niri-Rahmen exportieren
            crate::modules::exporter::export_niri(&new_theme);
            
        } else {
            println!("⚠️ Fehler beim Parsen der theme.toml. Syntax prüfen!");
        }
    }
}

// NEU: Scanner für verfügbare Themes
pub fn get_available_themes() -> Vec<String> {
    let mut themes = Vec::new();
    
    // Baue den Pfad zu ~/.config/niri-quickshell/themes
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
    path.push("niri-quickshell/themes");

    // Falls der Ordner noch nicht existiert, erstellen wir ihn stillschweigend
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }

    // Lese alle Dateien im Ordner aus
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    let file_name = entry.file_name();
                    let name_str = file_name.to_string_lossy();
                    
                    // Nur .toml Dateien akzeptieren
                    if name_str.ends_with(".toml") {
                        // ".toml" abschneiden und zur Liste hinzufügen
                        themes.push(name_str.replace(".toml", ""));
                    }
                }
            }
        }
    }
    
    // Alphabetisch sortieren, damit das UI später aufgeräumt aussieht
    themes.sort();
    themes
}
