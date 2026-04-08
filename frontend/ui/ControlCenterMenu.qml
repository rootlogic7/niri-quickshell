import QtQuick
import Quickshell

PopupWindow {
    id: rootPopup
    property var backend
    property var mainWindow 

    visible: false
    anchor.window: mainWindow
    anchor.rect.x: (mainWindow.width - width) / 2
    anchor.rect.y: Theme.barHeight 
    
    implicitWidth: 300
    implicitHeight: 200
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

            // App Launcher
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
                        backend.launchMenu();
                        rootPopup.visible = false; 
                    }
                }
            }

            // Theme Engine
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
                        rootPopup.visible = false;
                    }
                }
            }
        }
    }
}
