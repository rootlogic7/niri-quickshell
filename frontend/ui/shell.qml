import QtQuick
import Quickshell
import NiriState 1.0 // Das ist unser C++ Plugin!

PanelWindow {
    id: root
    
    // Verankere die Bar oben am Bildschirm (Wayland Layer Shell)
    anchors {
        top: true
        left: true
        right: true
    }
    height: 40
    color: "#1e1e2e" // Ein schickes, dunkles Grau (Catppuccin Theme)

    // Unser C++ Socket Reader im Hintergrund
    SocketReader {
        id: niriReader
    }

    // Zentrierte Anzeige der Workspaces
    Row {
        anchors.centerIn: parent
        spacing: 12

        // Loopt durch unsere FlatBuffers-Daten!
        Repeater {
            model: niriReader.workspaces

            Rectangle {
                width: 120
                implicitHeight: 28
                radius: 6
                // Wenn der Workspace aktiv ist (is_active == true), mach ihn blau, sonst dunkelgrau
                color: modelData.is_active ? "#89b4fa" : "#313244"

                Text {
                    anchors.centerIn: parent
                    text: modelData.name
                    font.pixelSize: 14
                    font.bold: modelData.is_active
                    color: modelData.is_active ? "#1e1e2e" : "#cdd6f4"
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        // Hier rufen wir unsere C++ Funktion auf!
                        niriReader.focusWorkspace(modelData.id)
                    }
                }
            }
        }
    }
}
