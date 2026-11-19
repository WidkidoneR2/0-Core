import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

// INT-330. Reads `core intent list --json` and draws it. Writes nothing.
// LEDGER_FIXTURE, when set, is read instead of core. r refreshes. Poll is 10s.
ShellRoot {
    id: root

    readonly property color bg: "#111622"
    readonly property color card: "#20293a"
    readonly property color divider: "#384050"
    readonly property color heading: "#90d0f0"
    readonly property color muted: "#98a0b0"
    readonly property color doneColor: "#78d888"
    readonly property color plannedColor: "#f0c048"
    readonly property color progressColor: "#70a0e8"
    readonly property color failedColor: "#e07870"
    readonly property color liveFg: "#58a860"
    readonly property color liveBg: "#103020"

    property string fixturePath: Quickshell.env("LEDGER_FIXTURE") || ""
    property var ledger: null
    property string errorText: ""
    property var rows: []

    function coreBin() {
        var home = Quickshell.env("HOME") || ""
        if (home.length > 0)
            return home + "/.local/bin/core"
        return "core"
    }

    function refresh() {
        if (reader.running)
            return
        reader.running = true
    }

    function applyText(raw) {
        try {
            root.ledger = JSON.parse(raw)
            root.errorText = ""
            root.rebuild()
        } catch (e) {
            root.errorText = "the ledger JSON could not be read"
        }
    }

    function rebuild() {
        var out = []
        if (!root.ledger)
            return
        var intents = root.ledger.intents || []
        var recent = root.ledger.recent_closed || []
        if (intents.length > 0)
            out.push({ kind: "head", text: "OPEN" })
        for (var i = 0; i < intents.length; i++)
            out.push({ kind: "open", data: intents[i] })
        if (recent.length > 0)
            out.push({ kind: "head", text: "CLOSED" })
        for (var j = 0; j < recent.length; j++)
            out.push({ kind: "closed", data: recent[j] })
        root.rows = out
    }

    function rowOf(item) {
        if (!item || !item.data)
            return {}
        return item.data
    }

    function padId(id) {
        var s = "" + (id === null || id === undefined ? "" : id)
        while (s.length < 3)
            s = "0" + s
        return s
    }

    function statusWord(row) {
        if (!row)
            return ""
        if (row.blocked)
            return "BLOCKED"
        var s = (row.status || "").toLowerCase()
        if (s === "in-progress")
            return "IN PROGRESS"
        if (s === "complete")
            return "DONE"
        return (row.status || "").toUpperCase()
    }

    function statusColor(row) {
        if (!row)
            return root.muted
        if (row.blocked)
            return root.failedColor
        var s = (row.status || "").toLowerCase()
        if (s === "complete" || s === "done")
            return root.doneColor
        if (s === "planned" || s === "pending")
            return root.plannedColor
        if (s === "in-progress")
            return root.progressColor
        if (s === "cancelled" || s === "canceled")
            return root.muted
        if (s === "failed")
            return root.failedColor
        return root.muted
    }

    function gatesText(row) {
        if (!row || !row.gates)
            return ""
        return row.gates.done + "/" + row.gates.total
    }

    function countText(name) {
        if (!root.ledger || !root.ledger.counts)
            return "—"
        var v = root.ledger.counts[name]
        if (v === null || v === undefined)
            return "—"
        return "" + v
    }

    function footerText() {
        if (!root.ledger)
            return ""
        var last = root.ledger.last_close
        var lastText = "last close unknown"
        if (last)
            lastText = "last close " + root.padId(last.id) + "  " + (last.status || "") + "  " + (last.closed || "")
        var tree = root.ledger.tree || {}
        var clean = tree.clean === true ? "clean" : tree.clean === false ? "dirty" : "clean unknown"
        var pushed = tree.pushed === true ? "pushed" : tree.pushed === false ? "unpushed" : "pushed unknown"
        return lastText + "   ·   " + clean + "   ·   " + pushed
    }

    Process {
        id: reader
        command: root.fixturePath.length > 0
                 ? ["cat", root.fixturePath]
                 : [root.coreBin(), "intent", "list", "--json"]
        stdout: StdioCollector {
            onStreamFinished: root.applyText(text)
        }
    }

    Component.onCompleted: root.refresh()

    Timer {
        interval: 10000
        running: true
        repeat: true
        onTriggered: root.refresh()
    }

    FloatingWindow {
        id: win
        title: "Intent ledger"
        color: "transparent"
        surfaceFormat.opaque: false
        implicitWidth: 560
        implicitHeight: 680

        Shortcut {
            sequence: "r"
            onActivated: root.refresh()
        }
        Shortcut {
            sequence: "R"
            onActivated: root.refresh()
        }

        Rectangle {
            anchors.fill: parent
            anchors.margins: 10
            radius: 18
            color: root.bg
            border.color: root.divider
            border.width: 1

            Flickable {
                id: scroller
                anchors.fill: parent
                anchors.margins: 18
                contentWidth: width
                contentHeight: body.implicitHeight
                clip: true
                boundsBehavior: Flickable.StopAtBounds

                Column {
                    id: body
                    width: scroller.width
                    spacing: 14

                    RowLayout {
                        width: parent.width

                        Text {
                            text: "INTENT LEDGER"
                            color: root.heading
                            font.pixelSize: 22
                            font.bold: true
                            font.letterSpacing: 1.2
                            Layout.fillWidth: true
                        }

                        Rectangle {
                            visible: root.ledger !== null
                            radius: 11
                            color: root.liveBg
                            implicitWidth: liveLabel.implicitWidth + 18
                            implicitHeight: 22

                            Text {
                                id: liveLabel
                                anchors.centerIn: parent
                                text: "LIVE"
                                color: root.liveFg
                                font.pixelSize: 11
                                font.bold: true
                                font.letterSpacing: 1
                            }
                        }
                    }

                    Text {
                        visible: root.errorText.length > 0
                        text: root.errorText
                        color: root.failedColor
                        font.pixelSize: 13
                    }

                    Rectangle {
                        width: parent.width
                        height: 1
                        color: root.divider
                    }

                    Row {
                        width: parent.width
                        spacing: 10

                        Repeater {
                            model: [
                                { n: root.countText("total"), label: "Total", color: "white" },
                                { n: root.countText("done"), label: "Done", color: root.doneColor },
                                { n: root.countText("planned"), label: "Pending", color: root.plannedColor }
                            ]
                            delegate: Rectangle {
                                width: (body.width - 20) / 3
                                height: 84
                                radius: 10
                                color: root.card

                                Text {
                                    anchors.left: parent.left
                                    anchors.leftMargin: 14
                                    anchors.top: parent.top
                                    anchors.topMargin: 12
                                    text: modelData.n
                                    color: modelData.color
                                    font.pixelSize: 32
                                    font.bold: true
                                }
                                Text {
                                    anchors.left: parent.left
                                    anchors.leftMargin: 14
                                    anchors.bottom: parent.bottom
                                    anchors.bottomMargin: 12
                                    text: modelData.label
                                    color: root.muted
                                    font.pixelSize: 13
                                }
                            }
                        }
                    }

                    Repeater {
                        model: root.rows
                        delegate: Column {
                            width: body.width
                            spacing: 4

                            Text {
                                visible: modelData.kind === "head"
                                text: modelData.text
                                color: root.heading
                                font.pixelSize: 13
                                font.bold: true
                                font.letterSpacing: 1.1
                                topPadding: 6
                            }

                            Rectangle {
                                visible: modelData.kind === "head"
                                width: parent.width
                                height: 1
                                color: root.divider
                            }

                            Row {
                                visible: modelData.kind !== "head"
                                width: parent.width
                                spacing: 10

                                Text {
                                    width: 36
                                    text: root.padId(root.rowOf(modelData).id)
                                    color: root.muted
                                    font.pixelSize: 14
                                }
                                Column {
                                    width: parent.width - 46
                                    spacing: 2

                                    Row {
                                        width: parent.width
                                        Text {
                                            width: parent.width - statusBlock.implicitWidth - 8
                                            text: root.rowOf(modelData).title || ""
                                            color: "white"
                                            font.pixelSize: 15
                                            font.bold: true
                                            elide: Text.ElideRight
                                        }
                                        Row {
                                            id: statusBlock
                                            spacing: 8
                                            Text {
                                                text: root.statusWord(root.rowOf(modelData))
                                                color: root.statusColor(root.rowOf(modelData))
                                                font.pixelSize: 12
                                                font.bold: true
                                            }
                                            Text {
                                                visible: modelData.kind === "open" && root.gatesText(root.rowOf(modelData)).length > 0
                                                text: root.gatesText(root.rowOf(modelData))
                                                color: root.muted
                                                font.pixelSize: 12
                                            }
                                        }
                                    }

                                    Text {
                                        width: parent.width
                                        visible: modelData.kind === "open" && (root.rowOf(modelData).vision || "").length > 0
                                        text: root.rowOf(modelData).vision || ""
                                        color: root.muted
                                        font.pixelSize: 13
                                        wrapMode: Text.WordWrap
                                    }
                                    Text {
                                        width: parent.width
                                        visible: modelData.kind === "closed"
                                        text: root.rowOf(modelData).closed || ""
                                        color: root.muted
                                        font.pixelSize: 13
                                    }
                                }
                            }
                        }
                    }

                    Rectangle {
                        width: parent.width
                        height: 1
                        color: root.divider
                    }

                    Text {
                        width: parent.width
                        text: root.footerText()
                        color: root.muted
                        font.pixelSize: 12
                        wrapMode: Text.WordWrap
                    }
                    Text {
                        text: "r refreshes"
                        color: root.divider
                        font.pixelSize: 11
                    }
                }
            }
        }
    }
}
