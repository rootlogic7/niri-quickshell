pragma Singleton
import QtQuick

QtObject {
    // --- FARBEN (Standard: Catppuccin Macchiato) ---
    property color bg: "#1e1e2e"
    property color bgHover: "#313244"
    property color text: "#cdd6f4"
    property color textDark: "#1e1e2e"
    property color primary: "#89b4fa"
    property color success: "#a6e3a1"
    property color error: "#f38ba8"

    // --- TYPOGRAFIE & ABMESSUNGEN ---
    property int fontSize: 14
    property int iconSize: 16
    property int barHeight: 40
    property int radius: 6
    property int margin: 16
    
    // --- ANIMATIONEN ---
    property int animDuration: 150 
}
