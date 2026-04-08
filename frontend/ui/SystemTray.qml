import QtQuick
import Quickshell

Row {
    property var backend
    property var mainWindow // Wird für das Popup-Anchoring benötigt

    spacing: 16

    Text {
        text: backend.networkName === "Offline" ? "⚠️ Offline" : "📶 " + backend.networkName
        color: backend.networkName === "Offline" ? Theme.error : Theme.text
        font.pixelSize: Theme.fontSize
        font.bold: true
        Behavior on color { ColorAnimation { duration: Theme.animDuration } }
    }

    Text {
        text: (backend.audioMuted ? "🔇 " : "🔊 ") + backend.audioVolume + "%"
        color: backend.audioMuted ? Theme.error : Theme.primary
        font.pixelSize: Theme.fontSize
        font.bold: true
        Behavior on color { ColorAnimation { duration: Theme.animDuration } }

        MouseArea {
            anchors.fill: parent
            cursorShape: Qt.PointingHandCursor
            onClicked: backend.toggleAudioMute()
        }
    }

    Text {
        text: "🔋 " + backend.batteryPercent + "%"
        color: Theme.success
        font.pixelSize: Theme.fontSize
        font.bold: true
    }

    // Uhrzeit & Kalender
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
            anchor.window: mainWindow
            anchor.rect.x: mainWindow.width - width - Theme.margin
            anchor.rect.y: Theme.barHeight + 8 
            
            implicitWidth: 220
            implicitHeight: 100 
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
