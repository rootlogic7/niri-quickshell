// frontend/ui/ControlCenterMenu.qml
import QtQuick
import Quickshell

PopupWindow {
    id: rootPopup
    property var backend
    property var mainWindow 

    // Positionierung: Ganz links, direkt unter der Bar
    anchor.window: mainWindow
    anchor.rect.x: 0 
    anchor.rect.y: Theme.barHeight 
    
    implicitWidth: Theme.menuWidth
    implicitHeight: 450 // Etwas höher für die Liste
    visible: false
    color: "transparent"

    // Interner Status: "main" oder "themes"
    property string mode: "main"

    Rectangle {
        anchors.fill: parent
        color: Theme.bg
        border.color: Theme.bgHover
        border.width: 1

        // Haupt-Layout
        Column {
            anchors { fill: parent; margins: 16 }
            spacing: 12

            // Header mit Zurück-Button
            Row {
                width: parent.width
                spacing: 10
                
                Rectangle {
                    width: 30; height: 30; radius: 4
                    color: Theme.bgHover
                    visible: rootPopup.mode === "themes"
                    Text { anchors.centerIn: parent; text: "←"; color: Theme.text }
                    MouseArea {
                        anchors.fill: parent
                        onClicked: rootPopup.mode = "main"
                    }
                }

                Text {
                    text: rootPopup.mode === "main" ? "System Menu" : "Select Theme"
                    color: Theme.text
                    font.pixelSize: 18
                    font.bold: true
                    verticalAlignment: Text.AlignVCenter
                    height: 30
                }
            }

            Rectangle { width: parent.width; height: 1; color: Theme.bgHover }

            // --- SEITE 1: HAUPTMENÜ ---
            Column {
                width: parent.width
                spacing: 8
                visible: rootPopup.mode === "main"

                MenuButton { 
                    icon: "🚀"; label: "App Launcher" 
                    onClicked: { backend.launchMenu(); rootPopup.visible = false }
                }

                MenuButton { 
                    icon: "🎨"; label: "Theme Engine" 
                    onClicked: rootPopup.mode = "themes"
                }
            }

            // --- SEITE 2: THEME LISTE ---
            ListView {
                width: parent.width
                height: 340
                visible: rootPopup.mode === "themes"
                model: backend.availableThemes
                spacing: 4
                clip: true

                delegate: Rectangle {
                    width: parent.width
                    height: 40
                    radius: Theme.radius
                    color: themeMA.containsMouse ? Theme.bgHover : "transparent"
                    
                    Text {
                        anchors { left: parent.left; leftMargin: 12; verticalCenter: parent.verticalCenter }
                        text: "󰸉  " + modelData
                        color: Theme.text
                        font.pixelSize: Theme.fontSize
                    }

                    MouseArea {
                        id: themeMA
                        anchors.fill: parent
                        hoverEnabled: true
                        onClicked: {
                            backend.setTheme(modelData)
                            // Optional: Menü nach Auswahl schließen
                            // rootPopup.visible = false 
                        }
                    }
                }
            }
        }
    }

    // Hilfskomponente für die Buttons (um Code-Duplikate zu vermeiden)
    component MenuButton : Rectangle {
        property string icon: ""
        property string label: ""
        signal clicked()

        width: parent.width; height: 45; radius: Theme.radius
        color: ma.containsMouse ? Theme.bgHover : "transparent"
        Behavior on color { ColorAnimation { duration: Theme.animDuration } }

        Row {
            anchors { left: parent.left; leftMargin: 12; verticalCenter: parent.verticalCenter }
            spacing: 12
            Text { text: icon; font.pixelSize: 18 }
            Text { text: label; color: Theme.text; font.pixelSize: Theme.fontSize }
        }

        MouseArea {
            id: ma
            anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
            onClicked: parent.clicked()
        }
    }
}
