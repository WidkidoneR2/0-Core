import Quickshell
import Quickshell.Io
import QtQuick

ShellRoot {
    PanelWindow {
        id: bar
        anchors { top: true; left: true; right: true }
        implicitHeight: 26
        color: "#141816"

        property var tags: []

        Rectangle {
            anchors.fill: parent
            color: "#141816"
        }

        FileView {
            id: wsFile
            path: "/home/christian/.cache/faelight/workspaces"
            watchChanges: true
            onFileChanged: reload()
            onLoaded: {
                try { bar.tags = JSON.parse(wsFile.text()).tags }
                catch (e) { console.log("parse failed: " + e) }
            }
        }

        Row {
            anchors.centerIn: parent
            spacing: 10

            Repeater {
                model: bar.tags
                Text {
                    text: modelData.id + 1
                    font.family: "JetBrainsMono Nerd Font"
                    font.pixelSize: 13
                    font.bold: modelData.selected
                    color: modelData.selected ? "#39ff14"
                         : modelData.occupied ? "#d7e0da"
                         : "#3c4641"
                }
            }
        }
    }
}
