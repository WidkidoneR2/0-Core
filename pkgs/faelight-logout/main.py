import sys, os, subprocess
try:
    import gi
    gi.require_version("Gtk", "4.0")
    gi.require_version("Gtk4LayerShell", "1.0")
    from gi.repository import Gtk, Gtk4LayerShell as LS, GLib
except Exception as e:
    print("IMPORT FAIL:", e); sys.exit(1)

DRY = "--dry-run" in sys.argv

CSS = """
window { background-color: rgba(8,13,8,0.94); }
.title { color:#C0DD97; font-size:26px; font-weight:bold; }
.hint  { color:#5f7f5f; font-size:13px; }
.tile-label { font-size:16px; font-weight:bold; }
button.tile { min-width:150px; min-height:150px; background-image:none; border-radius:22px; padding:0; outline:none; }
.shut { background-color:#1a0a11; border:2px solid #ED93B1; color:#ED93B1; box-shadow:0 0 14px 1px rgba(237,147,177,0.45), inset 0 0 10px rgba(237,147,177,0.18); }
.shut:hover, .shut.selected { border:3px solid #F4C0D1; box-shadow:0 0 30px 3px rgba(237,147,177,0.9), inset 0 0 14px rgba(237,147,177,0.30); }
.reb { background-color:#06140f; border:2px solid #5DCAA5; color:#5DCAA5; box-shadow:0 0 14px 1px rgba(93,202,165,0.45), inset 0 0 10px rgba(93,202,165,0.18); }
.reb:hover, .reb.selected { border:3px solid #9FE1CB; box-shadow:0 0 30px 3px rgba(93,202,165,0.9), inset 0 0 14px rgba(93,202,165,0.30); }
.out { background-color:#0d0a1a; border:2px solid #AFA9EC; color:#AFA9EC; box-shadow:0 0 14px 1px rgba(175,169,236,0.45), inset 0 0 10px rgba(175,169,236,0.18); }
.out:hover, .out.selected { border:3px solid #CECBF6; box-shadow:0 0 30px 3px rgba(175,169,236,0.9), inset 0 0 14px rgba(175,169,236,0.30); }
.lock { background-color:#0c1404; border:2px solid #97C459; color:#C0DD97; box-shadow:0 0 14px 1px rgba(151,196,89,0.45), inset 0 0 10px rgba(151,196,89,0.18); }
.lock:hover, .lock.selected { border:3px solid #C0DD97; box-shadow:0 0 30px 3px rgba(151,196,89,0.9), inset 0 0 14px rgba(151,196,89,0.30); }
.lbl-shut { color:#F4C0D1; } .lbl-reb { color:#9FE1CB; } .lbl-out { color:#CECBF6; } .lbl-lock { color:#C0DD97; }
"""

ITEMS = [
    (0,0,"Shutdown","system-shutdown-symbolic","shut"),
    (0,1,"Reboot","system-reboot-symbolic","reb"),
    (1,0,"Logout","system-log-out-symbolic","out"),
    (1,1,"Lock","system-lock-screen-symbolic","lock"),
]

def command_for(name):
    if name == "Shutdown": return ["systemctl","poweroff"], "Shutdown"
    if name == "Reboot":   return ["systemctl","reboot"], "Reboot"
    if name == "Lock":     return ["faelight-lock"], "Lock"
    if os.environ.get("PINNACLE_SOCKET"): return ["pinnacle","quit"], "Logout (pinnacle)"
    sid = os.environ.get("XDG_SESSION_ID")
    if sid: return ["loginctl","terminate-session",sid], "Logout (session "+sid+")"
    return ["loginctl","terminate-user",os.environ.get("USER","")], "Logout (user "+os.environ.get("USER","?")+")"

def on_activate(app):
    win = Gtk.ApplicationWindow(application=app)
    LS.init_for_window(win)
    LS.set_layer(win, LS.Layer.OVERLAY)
    for e in (LS.Edge.TOP, LS.Edge.BOTTOM, LS.Edge.LEFT, LS.Edge.RIGHT):
        LS.set_anchor(win, e, True)
    LS.set_keyboard_mode(win, LS.KeyboardMode.EXCLUSIVE)

    prov = Gtk.CssProvider(); prov.load_from_string(CSS)
    Gtk.StyleContext.add_provider_for_display(win.get_display(), prov, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION)

    root = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=20)
    root.set_halign(Gtk.Align.CENTER); root.set_valign(Gtk.Align.CENTER)
    title = Gtk.Label(label="Leave the forest" + ("   [dry-run]" if DRY else "")); title.add_css_class("title")
    root.append(title)

    grid = Gtk.Grid(); grid.set_row_spacing(30); grid.set_column_spacing(30); grid.set_halign(Gtk.Align.CENTER)
    tiles = {}
    def trigger(name):
        cmd, desc = command_for(name)
        print(("DRY-RUN: " if DRY else "RUN: ") + desc + " -> " + " ".join(cmd)); sys.stdout.flush()
        if not DRY:
            try: subprocess.Popen(cmd)
            except Exception as ex: print("EXEC FAIL:", ex); sys.stdout.flush()
            app.quit()
    for (r,c,label,icon,cls) in ITEMS:
        col = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12); col.set_halign(Gtk.Align.CENTER)
        btn = Gtk.Button(); btn.add_css_class("tile"); btn.add_css_class(cls); btn.set_can_focus(False)
        img = Gtk.Image.new_from_icon_name(icon); img.set_pixel_size(58); btn.set_child(img)
        def mk(n): return lambda _b: trigger(n)
        btn.connect("clicked", mk(label))
        lab = Gtk.Label(label=label); lab.add_css_class("tile-label"); lab.add_css_class("lbl-"+cls)
        col.append(btn); col.append(lab)
        grid.attach(col, c, r, 1, 1)
        tiles[(r,c)] = (btn, label)
    root.append(grid)

    hint = Gtk.Label(label="Hover or arrow to select   -   Enter to choose   -   Esc to cancel"); hint.add_css_class("hint")
    root.append(hint); win.set_child(root)

    sel = {"pos": None}
    def set_sel(pos):
        for p,(b,_n) in tiles.items():
            (b.add_css_class if p == pos else b.remove_css_class)("selected")
        sel["pos"] = pos
    def kbd_move(dr, dc):
        if sel["pos"] is None:
            set_sel((1,1)); return
        r,c = sel["pos"]
        set_sel((min(1,max(0,r+dr)), min(1,max(0,c+dc))))

    kc = Gtk.EventControllerKey()
    kc.set_propagation_phase(Gtk.PropagationPhase.CAPTURE)
    def on_key(ctrl, keyval, keycode, mods):
        if keyval == 0xff1b: app.quit(); return True
        if keyval == 0xff51: kbd_move(0,-1); return True
        if keyval == 0xff53: kbd_move(0, 1); return True
        if keyval == 0xff52: kbd_move(-1,0); return True
        if keyval == 0xff54: kbd_move( 1,0); return True
        if keyval in (0xff0d, 0x20):
            if sel["pos"] is not None: trigger(tiles[sel["pos"]][1])
            return True
        return False
    kc.connect("key-pressed", on_key)
    win.add_controller(kc)

    win.present()
    GLib.timeout_add_seconds(300, app.quit)
    print("faelight-logout " + ("[DRY-RUN]" if DRY else "[ARMED]") + " up -- nothing pre-selected; Esc or 300s to close")

app = Gtk.Application(application_id="org.faelight.logout")
app.connect("activate", on_activate)
sys.exit(app.run(None))
