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
        property int health: 100
        property string clock: ""

        FileView {
            id: wsFile
            path: "/home/christian/.cache/faelight/workspaces"
            watchChanges: true
            onFileChanged: reload()
            onLoaded: { try { bar.tags = JSON.parse(wsFile.text()).tags } catch (e) {} }
        }

        FileView {
            id: healthFile
            path: "/home/christian/.cache/faelight/health-status"
            watchChanges: true
            onFileChanged: reload()
            onLoaded: { bar.health = parseInt(healthFile.text()) || 100 }
        }

        Timer {
            interval: 1000
            running: true
            repeat: true
            triggeredOnStart: true
            onTriggered: bar.clock = Qt.formatDateTime(new Date(), "ddd HH:mm")
        }

        // left: tags
        Row {
            anchors { left: parent.left; leftMargin: 12; verticalCenter: parent.verticalCenter }
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

        // right: health then clock
        Row {
            anchors { right: parent.right; rightMargin: 12; verticalCenter: parent.verticalCenter }
            spacing: 14

            Text {
                text: bar.health + "%"
                font.family: "JetBrainsMono Nerd Font"
                font.pixelSize: 13
                // same thresholds the GTK bar uses: 95 and 80
                color: bar.health >= 95 ? "#39ff14"
                     : bar.health >= 80 ? "#ffc832"
                     : "#ff5050"
            }

            Text {
                text: bar.clock
                font.family: "JetBrainsMono Nerd Font"
                font.pixelSize: 13
                color: "#d7e0da"
            }
        }
    }
}
