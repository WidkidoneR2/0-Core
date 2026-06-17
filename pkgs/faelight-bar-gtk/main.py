import sys, time
import gi
gi.require_version("Gtk", "4.0")
gi.require_version("Gtk4LayerShell", "1.0")
from gi.repository import Gtk, Gtk4LayerShell as LS, GLib

BAR_HEIGHT = 30

CSS = """
window { background-color: rgba(8,13,8,0.94); }
.bar { min-height: 30px; padding: 0 14px;
       font-family: "JetBrainsMono Nerd Font", monospace; font-size: 13px; }
.left   { color: #C0DD97; font-weight: bold; }
.center { color: #AFA9EC; }
.right  { color: #5DCAA5; }
"""

def on_activate(app):
    win = Gtk.ApplicationWindow(application=app)
    LS.init_for_window(win)
    LS.set_layer(win, LS.Layer.TOP)
    LS.set_anchor(win, LS.Edge.TOP, True)
    LS.set_anchor(win, LS.Edge.LEFT, True)
    LS.set_anchor(win, LS.Edge.RIGHT, True)
    LS.set_anchor(win, LS.Edge.BOTTOM, False)
    LS.auto_exclusive_zone_enable(win)
    LS.set_keyboard_mode(win, LS.KeyboardMode.NONE)

    prov = Gtk.CssProvider()
    prov.load_from_string(CSS)
    Gtk.StyleContext.add_provider_for_display(
        win.get_display(), prov, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION)

    bar = Gtk.CenterBox()
    bar.add_css_class("bar")
    bar.set_size_request(-1, BAR_HEIGHT)

    left = Gtk.Label(label="faelight forest"); left.add_css_class("left")
    center = Gtk.Label(label="phase 1 skeleton"); center.add_css_class("center")
    right = Gtk.Label(label=""); right.add_css_class("right")
    bar.set_start_widget(left)
    bar.set_center_widget(center)
    bar.set_end_widget(right)
    win.set_child(bar)

    def tick():
        right.set_text(time.strftime("%a %d %b  %H:%M"))
        return True
    tick()
    GLib.timeout_add_seconds(1, tick)

    win.present()
    print("faelight-bar-gtk [phase1] up -- top anchor, exclusive zone, keyboard NONE; Ctrl+C to stop")

app = Gtk.Application(application_id="org.faelight.bar")
app.connect("activate", on_activate)
sys.exit(app.run(None))
