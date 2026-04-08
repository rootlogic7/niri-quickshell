import QtQuick
import QtQuick.Controls
import Quickshell
import NiriState 1.0
import "."
// HINWEIS: Falls du hier noch einen speziellen Import für dein C++ Plugin hattest 
// (z.B. "import NiriState"), füge ihn hier oben wieder ein!

PanelWindow {
    id: root
    anchors { top: true; left: true; right: true }
    
    implicitHeight: Theme.barHeight
    color: Theme.bg

    SocketReader {
        id: niriReader
        onToggleCcSignalChanged: {
            controlCenterPopup.visible = !controlCenterPopup.visible
        }
    }

    Item {
        anchors.fill: parent

        // ==========================================
        // 📍 LINKS: Navigation (Menü & Workspaces)
        // ==========================================
        Row {
            anchors { left: parent.left; leftMargin: Theme.margin; verticalCenter: parent.verticalCenter }
            spacing: 8

            // Der Hub-Button (ohne Popup, nur Klick-Erkennung)
            Rectangle {
                width: 36; height: 28; radius: Theme.radius
                color: menuMouseArea.containsMouse || controlCenterPopup.visible ? Theme.primary : Theme.bgHover
                Behavior on color { ColorAnimation { duration: Theme.animDuration } }

                Text {
                    anchors.centerIn: parent
                    text: "" 
                    font.pixelSize: Theme.iconSize
                    color: menuMouseArea.containsMouse || controlCenterPopup.visible ? Theme.textDark : Theme.primary
                    Behavior on color { ColorAnimation { duration: Theme.animDuration } }
                }

                MouseArea {
                    id: menuMouseArea
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    // Öffnet das Popup in der Mitte!
                    onClicked: controlCenterPopup.visible = !controlCenterPopup.visible
                }
            }

            // Workspaces
            Repeater {
                model: niriReader.workspaces

                Rectangle {
                    width: 120; height: 28; radius: Theme.radius
                    color: modelData.is_active ? Theme.primary : Theme.bgHover
                    Behavior on color { ColorAnimation { duration: Theme.animDuration } }

                    Text {
                        anchors.centerIn: parent
                        text: modelData.name
                        font.pixelSize: Theme.fontSize
                        font.bold: modelData.is_active
                        color: modelData.is_active ? Theme.textDark : Theme.text
                        Behavior on color { ColorAnimation { duration: Theme.animDuration } }
                    }

                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: niriReader.focusWorkspace(modelData.id)
                    }
                }
            }
        }

        // ==========================================
        // 📍 MITTE: Aktiver Fenster-Titel & Control Center
        // ==========================================
        Text {
            anchors.centerIn: parent
            text: niriReader.activeWindowTitle !== "" ? niriReader.activeWindowTitle : "Niri Desktop"
            color: Theme.text
            font.pixelSize: 15
            font.bold: true
            width: Math.min(implicitWidth, parent.width / 2.5)
            elide: Text.ElideRight
            horizontalAlignment: Text.AlignHCenter
        }

        PopupWindow {
            id: controlCenterPopup
            visible: false
            
            anchor.window: root
            anchor.rect.x: (root.width - width) / 2
            anchor.rect.y: Theme.barHeight 
            
            width: 300
            height: 200
            color: "transparent"

            Rectangle {
                anchors.fill: parent
                color: Theme.bg
                radius: Theme.radius
                border.color: Theme.bgHover
                border.width: 1

                Rectangle {
                    anchors { top: parent.top; left: parent.left; right: parent.right }
                    height: Theme.radius
                    color: Theme.bg
                }

                Column {
                    anchors { fill: parent; margins: 12 }
                    spacing: 8

                    Text {
                        text: "Control Center"
                        color: Theme.text
                        font.pixelSize: Theme.fontSize
                        font.bold: true
                        padding: 4
                    }

                    Rectangle { width: parent.width; height: 1; color: Theme.bgHover } 

                    // Button 1: App Launcher
                    Rectangle {
                        width: parent.width; height: 40; radius: Theme.radius
                        color: launcherMouseArea.containsMouse ? Theme.bgHover : "transparent"
                        Behavior on color { ColorAnimation { duration: Theme.animDuration } }

                        Text {
                            anchors { left: parent.left; leftMargin: 12; verticalCenter: parent.verticalCenter }
                            text: "🚀  App Launcher"
                            color: Theme.text
                            font.pixelSize: Theme.fontSize
                        }

                        MouseArea {
                            id: launcherMouseArea
                            anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                niriReader.launchMenu();
                                controlCenterPopup.visible = false; 
                            }
                        }
                    }

                    // Button 2: Theme Engine
                    Rectangle {
                        width: parent.width; height: 40; radius: Theme.radius
                        color: themeMouseArea.containsMouse ? Theme.bgHover : "transparent"
                        Behavior on color { ColorAnimation { duration: Theme.animDuration } }

                        Text {
                            anchors { left: parent.left; leftMargin: 12; verticalCenter: parent.verticalCenter }
                            text: "🎨  Theme Engine"
                            color: Theme.text
                            font.pixelSize: Theme.fontSize
                        }

                        MouseArea {
                            id: themeMouseArea
                            anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                console.log("Theme Engine clicked!");
                                controlCenterPopup.visible = false;
                            }
                        }
                    }
                }
            }
        }

        // ==========================================
        // 📍 RECHTS: System & Metriken
        // ==========================================
        Row {
            anchors { right: parent.right; rightMargin: Theme.margin; verticalCenter: parent.verticalCenter }
            spacing: 16

            Text {
                text: niriReader.networkName === "Offline" ? "⚠️ Offline" : "📶 " + niriReader.networkName
                color: niriReader.networkName === "Offline" ? Theme.error : Theme.text
                font.pixelSize: Theme.fontSize
                font.bold: true
                Behavior on color { ColorAnimation { duration: Theme.animDuration } }
            }

            Text {
                text: (niriReader.audioMuted ? "🔇 " : "🔊 ") + niriReader.audioVolume + "%"
                color: niriReader.audioMuted ? Theme.error : Theme.primary
                font.pixelSize: Theme.fontSize
                font.bold: true
                Behavior on color { ColorAnimation { duration: Theme.animDuration } }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: niriReader.toggleAudioMute()
                }
            }

            Text {
                text: "🔋 " + niriReader.batteryPercent + "%"
                color: Theme.success
                font.pixelSize: Theme.fontSize
                font.bold: true
            }

            // Uhrzeit & Kalender Popup
            Text {
                id: clockText
                color: Theme.text
                font.pixelSize: Theme.fontSize
                font.bold: true

                Timer {
                    interval: 1000; running: true; repeat: true; triggeredOnStart: true 
                    onTriggered: clockText.text = new Date().toLocaleTimeString(Qt.locale(), "HH:mm:ss")
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: calendarPopup.visible = !calendarPopup.visible
                }

                PopupWindow {
                    id: calendarPopup
                    visible: false
                    
                    anchor.window: root
                    anchor.rect.x: root.width - width - Theme.margin
                    anchor.rect.y: Theme.barHeight + 8 
                    
                    width: 220
                    height: 100 
                    color: "transparent"

                    Rectangle {
                        anchors.fill: parent
                        color: Theme.bg
                        radius: Theme.radius
                        border.color: Theme.bgHover
                        border.width: 1

                        Text {
                            anchors.centerIn: parent
                            text: new Date().toLocaleDateString(Qt.locale(), "dddd,\nd. MMMM yyyy")
                            color: Theme.text
                            font.bold: true
                            font.pixelSize: Theme.fontSize
                            horizontalAlignment: Text.AlignHCenter
                        }
                    }
                }
            }
        }
    }
}
