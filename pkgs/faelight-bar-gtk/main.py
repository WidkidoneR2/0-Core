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

_cpu_prev = {"total": 0, "idle": 0}

def read_cpu():
    try:
        with open("/proc/stat") as f:
            parts = f.readline().split()
        vals = [int(x) for x in parts[1:]]
        idle = vals[3] + (vals[4] if len(vals) > 4 else 0)
        total = sum(vals)
        dt = total - _cpu_prev["total"]
        di = idle - _cpu_prev["idle"]
        _cpu_prev["total"], _cpu_prev["idle"] = total, idle
        if dt <= 0:
            return 0
        return max(0, min(100, int(round(100.0 * (dt - di) / dt))))
    except Exception:
        return 0

def read_ram():
    try:
        info = {}
        with open("/proc/meminfo") as f:
            for line in f:
                k, _, v = line.partition(":")
                if v:
                    info[k.strip()] = int(v.split()[0])
        total = info.get("MemTotal", 0)
        avail = info.get("MemAvailable", 0)
        if total <= 0:
            return 0
        return max(0, min(100, int(round(100.0 * (total - avail) / total))))
    except Exception:
        return 0

def read_battery():
    for bat in ("/sys/class/power_supply/BAT1", "/sys/class/power_supply/BAT0"):
        try:
            with open(bat + "/capacity") as f:
                pct = int(f.read().strip())
        except Exception:
            continue
        charging = False
        try:
            with open(bat + "/status") as f:
                charging = f.read().strip() == "Charging"
        except Exception:
            pass
        return pct, charging
    return None, False

def read_wifi():
    try:
        net = "/sys/class/net"
        for iface in os.listdir(net):
            if iface.startswith("wl"):
                try:
                    with open(net + "/" + iface + "/operstate") as f:
                        if f.read().strip() == "up":
                            return True
                except Exception:
                    continue
    except Exception:
        pass
    return False

def load_pct(v):
    return "green" if v < 60 else "amber" if v < 85 else "red"

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

    right = Gtk.Box(spacing=10)
    cpu_lbl = Gtk.Label()
    ram_lbl = Gtk.Label()
    bat_lbl = Gtk.Label()
    wifi_lbl = Gtk.Label()
    clock_lbl = Gtk.Label()
    s1 = Gtk.Label(label=chr(0xB7)); s1.add_css_class("dim")
    s2 = Gtk.Label(label=chr(0xB7)); s2.add_css_class("dim")
    s3 = Gtk.Label(label=chr(0xB7)); s3.add_css_class("dim")
    s4 = Gtk.Label(label=chr(0xB7)); s4.add_css_class("dim")
    for w in (cpu_lbl, s1, ram_lbl, s2, bat_lbl, s3, wifi_lbl, s4, clock_lbl):
        right.append(w)

    bar.set_start_widget(left)
    bar.set_center_widget(center_lbl)
    bar.set_end_widget(right)
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

        cpu = read_cpu()
        cpu_lbl.set_text("CPU %d%%" % cpu)
        recolor(cpu_lbl, load_pct(cpu))

        ram = read_ram()
        ram_lbl.set_text("RAM %d%%" % ram)
        recolor(ram_lbl, load_pct(ram))

        pct, charging = read_battery()
        if pct is None:
            bat_lbl.set_text("")
            recolor(bat_lbl, "dim")
        else:
            bat_lbl.set_text(("+%d%%" if charging else "%d%%") % pct)
            recolor(bat_lbl, "green" if (charging or pct > 40) else "amber" if pct > 15 else "red")

        wifi_up = read_wifi()
        wifi_lbl.set_text("wifi" if wifi_up else "wifi off")
        recolor(wifi_lbl, "cyan" if wifi_up else "dim")

        clock_lbl.set_text(time.strftime("%a %d  %H:%M"))
        recolor(clock_lbl, "dim")
        return True

    tick()
    GLib.timeout_add_seconds(2, tick)
    win.present()
    print("faelight-bar-gtk [phase2] up -- health/intent/git live on 2s tick; Ctrl+C to stop")

app = Gtk.Application(application_id="org.faelight.bar")
app.connect("activate", on_activate)
sys.exit(app.run(None))
