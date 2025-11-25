# 🌲 Faelight Forest - Brave Browser Theming

Custom CSS theming for Brave browser to match the Faelight Forest desktop environment.

## 🎨 Color Palette
```css
/* Forest Backgrounds */
--dark-forest:   #0f1c16
--forest-green:  #1a3a2d
--sage-green:    #2e6146

/* Accent Colors */
--cyan-accent:   #5bb7a5
--mint-green:    #8ed1a3
--lime-highlight: #c7df63

/* Alert Colors */
--coral-red:     #ff6b6b
--pink-red:      #ee5a6f
```

## 📦 Installation

### 1. Install Stylus Extension

1. Open Brave browser
2. Go to Chrome Web Store
3. Search for **Stylus**
4. Click "Add to Brave"

### 2. Install Faelight Forest Theme

1. Click **Stylus** extension icon
2. Click "Manage"
3. Click "Write new style"
4. Name: `Faelight Forest - New Tab`
5. Add URL pattern: `chrome://newtab`
6. Paste CSS code below
7. Click **Save**

## 🎯 New Tab Page Theme
```css
/* ═══════════════════════════════════════════════════════════
   🌲 Faelight Forest Theme - Brave New Tab
   ═══════════════════════════════════════════════════════════ */

body {
  background: linear-gradient(135deg, #0f1c16 0%, #1a3a2d 50%, #2e6146 100%) !important;
  color: #5bb7a5 !important;
}

/* Search box */
.search-box, input[type="search"], input[type="text"] {
  background: rgba(26, 58, 45, 0.8) !important;
  border: 2px solid #5bb7a5 !important;
  color: #8ed1a3 !important;
  border-radius: 12px !important;
  padding: 12px 20px !important;
}

input:focus {
  border-color: #c7df63 !important;
  box-shadow: 0 0 20px rgba(91, 183, 165, 0.3) !important;
}

/* Links and shortcuts */
a, .tile {
  background: rgba(46, 97, 70, 0.6) !important;
  color: #8ed1a3 !important;
  border: 1px solid #5bb7a5 !important;
  border-radius: 8px !important;
}

a:hover, .tile:hover {
  background: linear-gradient(135deg, #5bb7a5 0%, #2e6146 100%) !important;
  border-color: #c7df63 !important;
  transform: translateY(-2px) !important;
  box-shadow: 0 4px 20px rgba(91, 183, 165, 0.4) !important;
}

/* Icons */
.mv-tile, .most-visited {
  background: rgba(46, 97, 70, 0.5) !important;
  border: 1px solid #5bb7a5 !important;
}
```

## 💾 Backup Your Styles

Export your Stylus configuration:

1. Click **Stylus** icon → Manage
2. Click ⚙️ Settings icon
3. Click **Export**
4. Save to: `~/dotfiles/brave/stylus-backup.json`

**Restore:**
```bash
# In Stylus: Manage → Settings → Import
# Select: ~/dotfiles/brave/stylus-backup.json
```

## 🔮 Future Themes (Coming Soon)

- GitHub - Forest-themed code repos
- YouTube - Enchanted video player
- Google Search - Mystical search results
- Reddit - Forest social feeds
- Documentation sites - Themed code docs

## 📝 Notes

- Themes apply only to new tabs by default
- Each website needs its own style definition
- Changes take effect immediately
- Compatible with Brave's built-in dark mode

## 🐛 Troubleshooting

**Theme not applying?**
- Check Stylus is enabled (icon should be colored)
- Verify URL pattern matches: `chrome://newtab`
- Try disabling/re-enabling the style
- Hard refresh: Ctrl+Shift+R

**Colors look wrong?**
- Check Brave's theme: brave://settings/appearance
- Set to "Dark" or "Same as system"
- Disable other theme extensions

---

*Part of the Faelight Forest dotfiles collection*
