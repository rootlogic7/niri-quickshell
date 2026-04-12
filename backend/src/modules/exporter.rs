use std::fs;
use std::path::PathBuf;
use crate::modules::theme::ThemeConfig;

pub fn export_fuzzel(theme: &ThemeConfig) {
    let mut base_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
    base_dir.push("niri-quickshell/fuzzel");

    // Ordner erstellen, falls nicht vorhanden
    if !base_dir.exists() {
        let _ = fs::create_dir_all(&base_dir);
    }

    let config_file = base_dir.join("fuzzel.ini");
    let colors_file = base_dir.join("colors.ini");

    // 1. HAUPT-CONFIG: Nur erstellen, wenn sie fehlt!
    // So kannst du später z.B. die Schriftart ändern, ohne dass wir sie überschreiben.
    if !config_file.exists() {
        let default_config = r#"[main]
font=monospace:size=14
prompt="> "
terminal=ghostty
layer=overlay
lines=10
width=40
horizontal-pad=20
vertical-pad=15
inner-pad=5

# Lade die von Quickshell generierten Farben
include=~/.config/niri-quickshell/fuzzel/colors.ini
"#;
        let _ = fs::write(&config_file, default_config);
    }

    // 2. FARBEN: Bei jedem Theme-Wechsel überschreiben
    let bg = theme.bg_color.replace("#", "");
    let fg = theme.fg_color.replace("#", "");
    let accent = theme.accent_color.replace("#", "");
    
    // Fuzzel nutzt RRGGBBAA. Ich setze den Hintergrund auf "e6" (leicht transparent)
    let colors_content = format!(
r#"[colors]
background={}e6
text={}ff
match={}ff
selection={}ff
selection-text={}ff
border={}ff"#,
        bg, fg, accent, accent, bg, accent
    );

    let _ = fs::write(colors_file, colors_content);
}

pub fn export_ghostty(theme: &ThemeConfig) {
    let mut base_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
    base_dir.push("niri-quickshell/ghostty");

    // Ordner erstellen
    if !base_dir.exists() {
        let _ = fs::create_dir_all(&base_dir);
    }

    let colors_file = base_dir.join("colors");

    // Basis-Farben für Ghostty (Hintergrund, Vordergrund, Auswahl)
    let mut content = format!(
        "background = {}\nforeground = {}\nselection-background = {}\nselection-foreground = {}\n\n",
        theme.bg_color, theme.fg_color, theme.accent_color, theme.bg_color
    );

    // Alle 16 Farben in ein Array packen, um leicht darüber iterieren zu können
    let colors = [
        &theme.color0, &theme.color1, &theme.color2, &theme.color3,
        &theme.color4, &theme.color5, &theme.color6, &theme.color7,
        &theme.color8, &theme.color9, &theme.color10, &theme.color11,
        &theme.color12, &theme.color13, &theme.color14, &theme.color15,
    ];

    // Nur in die Datei schreiben, wenn die Farbe im Theme existiert
    for (index, color_option) in colors.iter().enumerate() {
        if let Some(color_hex) = color_option {
            content.push_str(&format!("palette = {}={}\n", index, color_hex));
        }
    }

    let _ = fs::write(colors_file, content);
}

pub fn export_niri(theme: &ThemeConfig) {
    let mut base_dir = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("~/.config"));
    base_dir.push("niri-quickshell/niri");

    if !base_dir.exists() {
        let _ = std::fs::create_dir_all(&base_dir);
    }

    let colors_file = base_dir.join("colors.kdl");
    
    // Niri erlaubt das Auslagern von bestimmten Blöcken.
    // Wir setzen die Rahmen (border) und den Fokus-Ring auf deine Akzentfarbe.
    let content = format!(
r#"layout {{
    focus-ring {{
        active-color "{}"
        inactive-color "{}"
    }}
    border {{
        active-color "{}"
        inactive-color "{}"
    }}
}}"#,
        theme.accent_color, theme.bg_color, theme.accent_color, theme.bg_color
    );

    let _ = std::fs::write(colors_file, content);
}
