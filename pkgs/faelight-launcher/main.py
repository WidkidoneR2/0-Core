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
.section { color: #7fae6f; font-size: 13px; font-weight: bold; padding: 12px 8px 4px 8px; letter-spacing: 2px; }
"""

# INT-084 Phase 1B: scan XDG .desktop files for real app discovery.
import glob, re

def _xdg_app_dirs():
    dirs = []
    data_home = os.environ.get("XDG_DATA_HOME") or os.path.expanduser("~/.local/share")
    dirs.append(os.path.join(data_home, "applications"))
    data_dirs = os.environ.get("XDG_DATA_DIRS") or "/usr/local/share:/usr/share"
    # NixOS profiles live under these; also the current system + home-manager profiles.
    data_dirs += ":" + os.path.expanduser("~/.nix-profile/share")
    data_dirs += ":/run/current-system/sw/share"
    for d in data_dirs.split(":"):
        if d: dirs.append(os.path.join(d, "applications"))
    # de-dup, keep order
    seen=set(); out=[]
    for d in dirs:
        rp=os.path.realpath(d)
        if rp not in seen: seen.add(rp); out.append(d)
    return out

def _parse_desktop(path):
    """Return (name, exec_cmd, no_display) from a .desktop [Desktop Entry], or None."""
    name=None; exec_cmd=None; no_display=False; is_app=True; in_entry=False; terminal=False
    try:
        with open(path, encoding="utf-8", errors="ignore") as fh:
            for line in fh:
                line=line.strip()
                if line.startswith("[") and line.endswith("]"):
                    in_entry = (line == "[Desktop Entry]")
                    continue
                if not in_entry: continue
                if line.startswith("Name=") and name is None: name=line[5:].strip()
                elif line.startswith("Exec=") and exec_cmd is None: exec_cmd=line[5:].strip()
                elif line.startswith("NoDisplay=") and line[10:].strip().lower()=="true": no_display=True
                elif line.startswith("Hidden=") and line[7:].strip().lower()=="true": no_display=True
                elif line.startswith("Type=") and line[5:].strip()!="Application": is_app=False
                elif line.startswith("Terminal=") and line[9:].strip().lower()=="true": terminal=True
    except Exception:
        return None
    if not name or not exec_cmd or no_display or not is_app:
        return None
    # Strip .desktop field codes (%f %F %u %U %i %c %k etc.) from Exec.
    exec_cmd = re.sub(r"%[fFuUickdDnNvm]", "", exec_cmd).strip()
    # Terminal apps need a terminal wrapper.
    if terminal:
        exec_cmd = "alacritty -e " + exec_cmd
    return (name, exec_cmd)

# Junk .desktop entries to hide (substring match on Name, case-insensitive).
_BLOCK = [
    "nixos manual", "remote viewer", "remote-viewer", "gvim", "vim",
    "neovim wrapper", "nvim", "avahi", "qv4l2", "qvidcap", "compose",
    "feh", "xterm", "uxterm", "bssh", "bvnc", ".desktop",
]
def _blocked(name):
    n = name.lower()
    return any(b in n for b in _BLOCK)

# Forest tools have NO .desktop files, so add them explicitly. Terminal tools get wrapped.
_FOREST_TOOLS = [
    ("Faelight FM",        "alacritty -e faelight-fm"),
    ("Faelight VM",        "alacritty -e vm status"),
    ("Faelight Lock",      "faelight-lock"),
    ("Faelight Update",    "alacritty -e faelight-update"),
    ("Faelight Vault",     "alacritty -e faelight-vault list"),
    ("Faelight Sandbox",   "alacritty -e faelight-sandbox status"),
    ("Faelight Clipboard", "faelight-clipboard pick"),
    ("Faelight Term",      "faelight-term"),
    ("Forest Shell",       "alacritty -e faelight-shell"),
    ("Forest Doctor",      "alacritty -e core doctor run"),
]

def discover_apps():
    found={}  # name -> exec (last wins, so user-local overrides system)
    for d in _xdg_app_dirs():
        for path in glob.glob(os.path.join(d, "*.desktop")):
            parsed=_parse_desktop(path)
            if parsed and not _blocked(parsed[0]):
                found[parsed[0]] = parsed[1]
    forest = sorted(_FOREST_TOOLS, key=lambda kv: kv[0].lower())
    scanned = sorted(found.items(), key=lambda kv: kv[0].lower())
    # Sectioned: ("header", title, "") or ("app", name, cmd).
    out=[]
    if forest:
        out.append(("header", "FOREST TOOLS", ""))
        out += [("app", n, c) for (n, c) in forest]
    if scanned:
        out.append(("header", "APPLICATIONS", ""))
        out += [("app", n, c) for (n, c) in scanned]
    return out or [("app", "(no apps found)", "true")]

APPS = discover_apps()

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
    scroll = Gtk.ScrolledWindow()
    scroll.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)  # vertical scroll only
    scroll.set_min_content_height(420)   # fixed list height -- window no longer grows
    scroll.set_max_content_height(420)
    scroll.set_child(listbox)
    outer.append(scroll)

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
        child = listbox.get_first_child()
        while child: listbox.remove(child); child = listbox.get_first_child()
        q = query.lower().strip()

        # Filter apps by query, then keep section headers only if their section has matches.
        flt=[]
        i=0
        while i < len(APPS):
            kind, name, cmd = APPS[i]
            if kind == "header":
                # collect this section's apps that match
                section=[]
                j=i+1
                while j < len(APPS) and APPS[j][0] == "app":
                    n2,c2 = APPS[j][1], APPS[j][2]
                    if not q or q in n2.lower(): section.append(("app", n2, c2))
                    j+=1
                if section:
                    flt.append(("header", name, ""))
                    flt += section
                i=j
            else:
                if not q or q in name.lower(): flt.append((kind, name, cmd))
                i+=1
        state["filtered"] = flt

        color=0
        for (kind, name, cmd) in flt:
            row = Gtk.ListBoxRow()
            lbl = Gtk.Label(label=name); lbl.set_halign(Gtk.Align.START)
            if kind == "header":
                lbl.add_css_class("section")
                row.set_selectable(False); row.set_activatable(False)
            else:
                lbl.add_css_class("row"); lbl.add_css_class("c" + str(color % 6)); color+=1
            row.set_child(lbl)
            listbox.append(row)

        # land selection on the first APP row (skip leading header)
        state["sel"] = next((k for k,e in enumerate(flt) if e[0]=="app"), 0)
        app_count = sum(1 for e in flt if e[0]=="app")
        count.set_label(f"{app_count} app(s)  -  Enter to launch  -  Esc to cancel")
        highlight()

    def highlight():
        rows=_rows()
        for i, row in enumerate(rows):
            lbl = row.get_child()
            (lbl.add_css_class if i == state["sel"] else lbl.remove_css_class)("selected")
        # Scroll the selected row into view so it is never hidden below the fold.
        if 0 <= state["sel"] < len(rows):
            rows[state["sel"]].grab_focus()

    def _rows():
        out=[]; r=listbox.get_first_child()
        while r: out.append(r); r=r.get_next_sibling()
        return out

    def move(d):
        flt=state["filtered"]; n=len(flt)
        if n==0: return
        i=state["sel"]
        for _ in range(n):                 # step until we land on an "app" row
            i=(i+d) % n
            if flt[i][0]=="app":
                state["sel"]=i; highlight(); return

    search.connect("changed", lambda e: rebuild(e.get_text()))

    kc = Gtk.EventControllerKey(); kc.set_propagation_phase(Gtk.PropagationPhase.CAPTURE)
    def on_key(ctrl, keyval, keycode, mods):
        if keyval == 0xff1b: app.quit(); return True              # Esc
        if keyval == 0xff52: move(-1); return True                # Up
        if keyval == 0xff54: move(1);  return True                # Down
        if keyval == 0xff0d:                                       # Enter
            flt=state["filtered"]
            if flt and 0 <= state["sel"] < len(flt) and flt[state["sel"]][0]=="app":
                launch(flt[state["sel"]][2])
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
