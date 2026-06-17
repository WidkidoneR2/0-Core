import sys, os, time, sqlite3, subprocess, tomllib
import gi
gi.require_version("Gtk", "4.0")
gi.require_version("Gtk4LayerShell", "1.0")
from gi.repository import Gtk, Gtk4LayerShell as LS, GLib

HOME = os.environ.get("HOME", "")
REPO = HOME + "/0-core"
DB = REPO + "/runtime/state.db"
FOCUS = HOME + "/.local/state/0-core/intent/focus.toml"
BAR_HEIGHT = 30
COLOR_CLASSES = ("green", "amber", "red", "purple", "cyan", "dim")

CSS = """
window { background-color: rgba(8,13,8,0.94); }
.bar { min-height: 30px; padding: 0 16px;
       font-family: "JetBrainsMono Nerd Font", monospace; font-size: 13px; }
label   { color: #D7E0DA; }
.dim    { color: #788C82; }
.green  { color: #39FF14; }
.amber  { color: #FFC832; }
.red    { color: #FF5050; }
.purple { color: #B482FF; font-weight: bold; }
.cyan   { color: #32DCFF; }
"""

def read_health():
    for path in ("/etc/faelight/HEALTH", HOME + "/.cache/faelight/health-status"):
        try:
            with open(path) as f:
                return int(f.read().strip().rstrip("%"))
        except Exception:
            continue
    return 100

def read_intent():
    try:
        with open(FOCUS, "rb") as f:
            t = (tomllib.load(f).get("title") or "").strip()
        return (t[:52] + "...") if len(t) > 55 else t
    except Exception:
        return ""

def read_friday():
    try:
        con = sqlite3.connect("file:" + DB + "?mode=ro", uri=True, timeout=0.5)
        cutoff = int(time.time()) - 300
        row = con.execute(
            "SELECT action, confidence FROM friday_patterns "
            "WHERE confidence >= 0.75 AND last_seen > ? "
            "ORDER BY confidence DESC LIMIT 1", (cutoff,)).fetchone()
        con.close()
        if row:
            return row[0]
    except Exception:
        pass
    return ""

def git(args):
    try:
        r = subprocess.run(["git"] + args, cwd=REPO,
                           capture_output=True, text=True, timeout=1.0)
        return r.stdout.strip()
    except Exception:
        return ""

def read_git():
    branch = git(["branch", "--show-current"]) or "main"
    clean = git(["status", "--porcelain"]) == ""
    return branch, clean

def recolor(widget, cls):
    for c in COLOR_CLASSES:
        widget.remove_css_class(c)
    if cls:
        widget.add_css_class(cls)

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

    left = Gtk.Box(spacing=10)
    health_lbl = Gtk.Label()
    sep = Gtk.Label(label=chr(0xB7)); sep.add_css_class("dim")
    git_lbl = Gtk.Label()
    left.append(health_lbl); left.append(sep); left.append(git_lbl)

    center_lbl = Gtk.Label()
    right_lbl = Gtk.Label()

    bar.set_start_widget(left)
    bar.set_center_widget(center_lbl)
    bar.set_end_widget(right_lbl)
    win.set_child(bar)

    def tick():
        h = read_health()
        health_lbl.set_text("H:%d%%" % h)
        recolor(health_lbl, "green" if h >= 95 else "amber" if h >= 80 else "red")

        branch, clean = read_git()
        git_lbl.set_text(branch if clean else branch + "*")
        recolor(git_lbl, "green" if clean else "amber")

        intent = read_intent()
        if intent:
            center_lbl.set_text(intent); recolor(center_lbl, "purple")
        else:
            fr = read_friday()
            center_lbl.set_text(fr); recolor(center_lbl, "cyan" if fr else "dim")

        right_lbl.set_text(time.strftime("%a %d  %H:%M"))
        recolor(right_lbl, "dim")
        return True

    tick()
    GLib.timeout_add_seconds(2, tick)
    win.present()
    print("faelight-bar-gtk [phase2] up -- health/intent/git live on 2s tick; Ctrl+C to stop")

app = Gtk.Application(application_id="org.faelight.bar")
app.connect("activate", on_activate)
sys.exit(app.run(None))
