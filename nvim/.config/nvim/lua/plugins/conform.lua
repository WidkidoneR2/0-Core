-- ═══════════════════════════════════════════════════════════
-- 🔧 Conform.nvim - Auto-formatting for Omarchy
-- Lightweight, async, professional code formatting
-- ═══════════════════════════════════════════════════════════

return {
	"stevearc/conform.nvim",
	event = { "BufReadPre", "BufNewFile" },
	opts = {
		formatters_by_ft = {
			-- Shell scripts (your dotfiles!)
			fish = { "fish_indent" },
			sh = { "shfmt" },
			bash = { "shfmt" },

			-- Config files
			toml = { "taplo" },
			yaml = { "prettier" },
			json = { "prettier" },

			-- Documentation
			markdown = { "prettier" },

			-- Web/JS (if you use it)
			javascript = { "prettier" },
			typescript = { "prettier" },
			html = { "prettier" },
			css = { "prettier" },

			-- Lua (for nvim configs)
			lua = { "stylua" },

			-- Python (if you use it)
			python = { "black" },
		},

		-- Format on save
		format_on_save = {
			-- Timeout for formatting
			timeout_ms = 500,
			-- Fallback to LSP if formatter not available
			lsp_fallback = true,
		},

		-- Keybinding to manually format
		-- You can use <leader>cf (Code Format)
	},

	keys = {
		{
			"<leader>cf",
			function()
				require("conform").format({ async = true, lsp_fallback = true })
			end,
			mode = "",
			desc = "Format buffer",
		},
	},
}
