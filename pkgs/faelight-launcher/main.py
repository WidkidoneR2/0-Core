import sys, os, subprocess
try:
    import gi
    gi.require_version("Gtk", "4.0")
    gi.require_version("Gtk4LayerShell", "1.0")
    from gi.repository import Gtk, Gtk4LayerShell as LS, GLib, Gdk
except Exception as e:
    print("IMPORT FAIL:", e); sys.exit(1)

# --float = compact centered panel (desktop visible). Default = fullscreen overlay.
FLOAT = "--fullscreen" not in sys.argv  # INT-084: float is the default; pass --fullscreen for overlay

# INT-084 faelight-launcher -- candy-neon GTK4 app launcher, faelight-logout-grade.
# Phase 1A: a themed window that appears with a search box + app list (placeholder apps).
# Pattern lifted from faelight-logout (INT-064): layer-shell overlay + CSS-as-string.

CSS = """
window { background-color: rgba(8,13,8,0.94); }
window.float { background-color: rgba(8,13,8,0.30); }   /* float: see the desktop through a light dim */
/* Kill GTK's default opaque list background so the glassy panel shows through. */
listbox, list, listview { background-color: transparent; background-image: none; }
listbox > row, row { background-color: transparent; background-image: none; }
listbox > row:selected, row:selected { background-color: transparent; }
scrolledwindow, viewport { background-color: transparent; }
.panel {
  background-color: rgba(8,13,8,0.55);   /* glassy -- desktop shows through */
  border: 2px solid #97C459; border-radius: 22px; padding: 22px;
  box-shadow: 0 0 40px 4px rgba(151,196,89,0.45), inset 0 0 18px rgba(151,196,89,0.10);
}
/* Per-app neon colors (cycled). Rows are transparent until selected; text glows its color. */
.c0 { color:#A6E22E; } .c0.selected { color:#0c1404; background-color:#A6E22E; box-shadow:0 0 22px 2px rgba(166,226,46,0.8); }
.c1 { color:#FF5C57; } .c1.selected { color:#1a0a11; background-color:#FF5C57; box-shadow:0 0 22px 2px rgba(255,92,87,0.8); }
.c2 { color:#36E0D0; } .c2.selected { color:#06140f; background-color:#36E0D0; box-shadow:0 0 22px 2px rgba(54,224,208,0.8); }
.c3 { color:#AFA9EC; } .c3.selected { color:#0d0a1a; background-color:#AFA9EC; box-shadow:0 0 22px 2px rgba(175,169,236,0.8); }
.c4 { color:#ED93B1; } .c4.selected { color:#1a0a11; background-color:#ED93B1; box-shadow:0 0 22px 2px rgba(237,147,177,0.8); }
.c5 { color:#F4D06F; } .c5.selected { color:#141004; background-color:#F4D06F; box-shadow:0 0 22px 2px rgba(244,208,111,0.8); }
.search {
  font-size: 22px; color: #C0DD97;
  background-color: #0c1404; border: 2px solid #97C459; border-radius: 16px;
  padding: 12px 18px; margin: 0 0 10px 0;
  box-shadow: 0 0 16px 1px rgba(151,196,89,0.40), inset 0 0 10px rgba(151,196,89,0.15);
}
.search:focus { border: 2px solid #C0DD97; box-shadow: 0 0 26px 2px rgba(151,196,89,0.75); }
.row {
  font-size: 17px; font-weight: bold; padding: 10px 16px; border-radius: 12px;
  background-color: transparent;
}
.count { color: #5f7f5f; font-size: 12px; margin-top: 8px; }
"""

# Placeholder apps for Phase 1A -- real .desktop scan comes in Phase 1B.
APPS = [
    ("Brave",    "brave"),
    ("Alacritty","alacritty"),
    ("Helix",    "alacritty -e hx"),
    ("Files",    "alacritty -e fm"),
    ("Firefox",  "firefox"),
    ("VM",       "alacritty -e vm status"),
    ("NixVim",   "alacritty -e sh -c 'cd ~/nixvim-play && nix run .# --accept-flake-config'"),
]

def on_activate(app):
    win = Gtk.ApplicationWindow(application=app)
    LS.init_for_window(win)
    LS.set_layer(win, LS.Layer.OVERLAY)
    if FLOAT:
        # Floating: anchor to nothing (compositor centers it); keep keyboard exclusive.
        win.add_css_class("float")
        LS.set_keyboard_mode(win, LS.KeyboardMode.EXCLUSIVE)
    else:
        for e in (LS.Edge.TOP, LS.Edge.BOTTOM, LS.Edge.LEFT, LS.Edge.RIGHT):
            LS.set_anchor(win, e, True)
        LS.set_keyboard_mode(win, LS.KeyboardMode.EXCLUSIVE)

    prov = Gtk.CssProvider(); prov.load_from_string(CSS)
    Gtk.StyleContext.add_provider_for_display(win.get_display(), prov, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION)

    # Centered column: search box on top, results list below.
    outer = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
    outer.set_halign(Gtk.Align.CENTER); outer.set_valign(Gtk.Align.CENTER)
    outer.set_size_request(560, -1)
    if FLOAT:
        outer.add_css_class("panel")

    search = Gtk.Entry(); search.add_css_class("search")
    search.set_placeholder_text("launch the forest...")
    outer.append(search)

    listbox = Gtk.ListBox(); listbox.set_selection_mode(Gtk.SelectionMode.NONE)
    outer.append(listbox)

    count = Gtk.Label(); count.add_css_class("count"); count.set_halign(Gtk.Align.START)
    outer.append(count)
    win.set_child(outer)

    state = {"rows": [], "sel": 0, "filtered": []}

    def launch(cmd):
        print("LAUNCH:", cmd); sys.stdout.flush()
        try: subprocess.Popen(cmd.split())
        except Exception as ex: print("EXEC FAIL:", ex)
        app.quit()

    def rebuild(query=""):
        # clear
        child = listbox.get_first_child()
        while child: listbox.remove(child); child = listbox.get_first_child()
        q = query.lower().strip()
        flt = [(n,c) for (n,c) in APPS if q in n.lower()] if q else list(APPS)
        state["filtered"] = flt
        for i, (name, cmd) in enumerate(flt):
            row = Gtk.ListBoxRow()
            lbl = Gtk.Label(label=name); lbl.set_halign(Gtk.Align.START)
            lbl.add_css_class("row"); lbl.add_css_class("c" + str(i % 6))
            row.set_child(lbl)
            listbox.append(row)
        state["sel"] = 0
        count.set_label(f"{len(flt)} app(s)  -  Enter to launch  -  Esc to cancel")
        highlight()

    def highlight():
        for i, row in enumerate(_rows()):
            lbl = row.get_child()
            (lbl.add_css_class if i == state["sel"] else lbl.remove_css_class)("selected")

    def _rows():
        out=[]; r=listbox.get_first_child()
        while r: out.append(r); r=r.get_next_sibling()
        return out

    def move(d):
        n=len(state["filtered"])
        if n==0: return
        state["sel"]=(state["sel"]+d) % n
        highlight()

    search.connect("changed", lambda e: rebuild(e.get_text()))

    kc = Gtk.EventControllerKey(); kc.set_propagation_phase(Gtk.PropagationPhase.CAPTURE)
    def on_key(ctrl, keyval, keycode, mods):
        if keyval == 0xff1b: app.quit(); return True              # Esc
        if keyval == 0xff52: move(-1); return True                # Up
        if keyval == 0xff54: move(1);  return True                # Down
        if keyval == 0xff0d:                                       # Enter
            if state["filtered"]:
                launch(state["filtered"][state["sel"]][1])
            return True
        return False
    kc.connect("key-pressed", on_key); win.add_controller(kc)

    win.present(); search.grab_focus()
    rebuild("")
    GLib.timeout_add_seconds(120, app.quit)
    print("faelight-launcher [Phase 1A] up -- type to filter, arrows + Enter, Esc to close")

app = Gtk.Application(application_id="org.faelight.launcher")
app.connect("activate", on_activate)
sys.exit(app.run(None))
