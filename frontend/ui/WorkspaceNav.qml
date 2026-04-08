import QtQuick

Row {
    // Diese Properties werden von außen (von der shell.qml) befüllt
    property var backend
    property var controlCenter

    spacing: 8

    // Der Hub-Button
    Rectangle {
        width: 36; height: 28; radius: Theme.radius
        color: menuMouseArea.containsMouse || controlCenter.visible ? Theme.primary : Theme.bgHover
        Behavior on color { ColorAnimation { duration: Theme.animDuration } }

        Text {
            anchors.centerIn: parent
            text: "" 
            font.pixelSize: Theme.iconSize
            color: menuMouseArea.containsMouse || controlCenter.visible ? Theme.textDark : Theme.primary
            Behavior on color { ColorAnimation { duration: Theme.animDuration } }
        }

        MouseArea {
            id: menuMouseArea
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: controlCenter.visible = !controlCenter.visible
        }
    }

    // Workspaces
    Repeater {
        model: backend.workspaces

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
                onClicked: backend.focusWorkspace(modelData.id)
            }
        }
    }
}
