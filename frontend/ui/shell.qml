import QtQuick
import Quickshell
import NiriState 1.0

PanelWindow {
    id: root

    // --- HIER IST DIE MAGIE GEGEN DAS QUADRAT ---
    // Wir verankern das Fenster fest oben, links und rechts am Monitor.
    anchors {
        top: true
        left: true
        right: true
    }

    // Wir sagen Wayland, dass die Leiste exakt 40 Pixel hoch sein soll.
    implicitHeight: 40

    // Eine Hintergrundfarbe für die Leiste
    color: "#1e1e2e"

    SocketReader {
        id: niriReader
    }

    // Ein Container-Item, das die ganze Leiste ausfüllt
    Item {
        anchors.fill: parent

        // 📍 LINKS: Navigation (Workspaces)
        Row {
            anchors {
                left: parent.left
                leftMargin: 16
                verticalCenter: parent.verticalCenter
            }
            spacing: 8

            Rectangle {
                width: 36
                height: 28
                radius: 6
                // Hover-Effekt: Wird beim Überfahren blau
                color: menuMouseArea.containsMouse ? "#89b4fa" : "#313244"

                Text {
                    anchors.centerIn: parent
                    //  ist das NixOS-Logo in Nerd Fonts. 
                    // Falls du ein Kästchen siehst, ändere es vorerst in "❄️"
                    text: "" 
                    font.pixelSize: 16
                    color: menuMouseArea.containsMouse ? "#1e1e2e" : "#89b4fa"
                }

                MouseArea {
                    id: menuMouseArea
                    anchors.fill: parent
                    hoverEnabled: true // Aktiviert den Hover-Effekt
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        //console.log("Hauptmenü geklickt!")
                        niriReader.launchMenu()
                    }
                }
            }

            Repeater {
                model: niriReader.workspaces

                Rectangle {
                    width: 120
                    implicitHeight: 28
                    radius: 6
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
                            niriReader.focusWorkspace(modelData.id)
                        }
                    }
                }
            }
        }

        // 📍 MITTE: Aktiver Fenster-Titel
        Text {
            anchors.centerIn: parent
            // Fallback-Text, wenn kein Fenster offen ist
            text: niriReader.activeWindowTitle !== "" ? niriReader.activeWindowTitle : "Niri Desktop"
            color: "#cdd6f4"
            font.pixelSize: 15
            font.bold: true

            // Sehr wichtig: Verhindert, dass ultralange Titel (z.B. YouTube) die Leiste sprengen!
            width: Math.min(implicitWidth, parent.width / 2.5)
            elide: Text.ElideRight
            horizontalAlignment: Text.AlignHCenter
        }

        // 📍 RECHTS: System & Metriken
        Row {
            anchors {
                right: parent.right
                rightMargin: 16
                verticalCenter: parent.verticalCenter
            }
            spacing: 16

            // --- NEU: Netzwerk/WLAN ---
            Text {
                text: niriReader.networkName === "Offline" ? "⚠️ Offline" : "📶 " + niriReader.networkName
                color: niriReader.networkName === "Offline" ? "#f38ba8" : "#cdd6f4"
                font.pixelSize: 14
                font.bold: true
            }

            // --- Audio ---
            Text {
                // Zeigt 🔇 wenn stummgeschaltet, sonst 🔊
                text: (niriReader.audioMuted ? "🔇 " : "🔊 ") + niriReader.audioVolume + "%"
                // Rot bei Stummschaltung, Blau im Normalbetrieb
                color: niriReader.audioMuted ? "#f38ba8" : "#89b4fa"
                font.pixelSize: 14
                font.bold: true
            }

            Text {
                text: "🔋 " + niriReader.batteryPercent + "%"
                color: "#a6e3a1"
                font.pixelSize: 14
                font.bold: true
            }

            Text {
                id: clockText
                color: "#cdd6f4"
                font.pixelSize: 14
                font.bold: true

                Timer {
                    interval: 1000
                    running: true
                    repeat: true
                    triggeredOnStart: true 
                    onTriggered: {
                        clockText.text = new Date().toLocaleTimeString(Qt.locale(), "HH:mm:ss")
                    }
                }
            }
        }
    }
}
