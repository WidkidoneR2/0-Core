-- ═══════════════════════════════════════════════════════════
-- 🌲 Yazi Init - Plugin Configuration
-- ═══════════════════════════════════════════════════════════

-- Smart-enter: Auto-drill single-item folders
require("smart-enter"):setup({
	open_multi = false,
})

-- Full-border: Better visual borders
require("full-border"):setup()

-- Git: Show git status in file list
require("git"):setup()
