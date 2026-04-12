import QtQuick
import Quickshell
import NiriState 1.0
import "."

PanelWindow {
    id: root
    anchors { top: true; left: true; right: true }
    
    implicitHeight: Theme.barHeight
    color: Theme.bg
    Behavior on color { ColorAnimation { duration: Theme.animDuration } }

    // Unser C++ Gehirn
    ShellStateStore {
        id: niriReader
        
        onToggleCcSignalChanged: {
            controlCenter.visible = !controlCenter.visible
        }

        // Die Theme-Magie ist hier jetzt sofort mit eingebaut!
        onThemeChanged: {
            Theme.bg = niriReader.themeBackground
            Theme.text = niriReader.themeForeground
            Theme.primary = niriReader.themeAccent
            Theme.bgHover = Qt.lighter(niriReader.themeBackground, 1.1)
        }
    }

    // Das versteckte Control-Center Popup
    ControlCenterMenu {
        id: controlCenter
        backend: niriReader
        mainWindow: root
        onVisibleChanged: if(!visible) mode = "main"
    }

    // Die eigentliche Statusleiste
    Item {
        anchors.fill: parent

        // 📍 LINKS
        WorkspacesNav {
            anchors { left: parent.left; leftMargin: Theme.margin; verticalCenter: parent.verticalCenter }
            backend: niriReader
            controlCenter: controlCenter
        }

        // 📍 MITTE
        Text {
            anchors.centerIn: parent
            text: niriReader.activeWindowTitle !== "" ? niriReader.activeWindowTitle : "Niri Desktop"
            color: Theme.text
            font.pixelSize: 15
            font.bold: true
            width: Math.min(implicitWidth, parent.width / 2.5)
            elide: Text.ElideRight
            horizontalAlignment: Text.AlignHCenter
            Behavior on color { ColorAnimation { duration: Theme.animDuration } }
        }

        // 📍 RECHTS
        SystemTray {
            anchors { right: parent.right; rightMargin: Theme.margin; verticalCenter: parent.verticalCenter }
            backend: niriReader
            mainWindow: root
        }
    }
}
