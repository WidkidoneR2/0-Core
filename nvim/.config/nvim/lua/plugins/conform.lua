-- ═══════════════════════════════════════════════════════════
-- 🔧 Conform.nvim - Auto-formatting for Neovim on Arch Linux
-- Lightweight, async, professional code formatting
-- LazyVim handles format_on_save automatically
-- ═══════════════════════════════════════════════════════════

local function is_executable(cmd)
	return vim.fn.executable(cmd) == 1
end

return {
	"stevearc/conform.nvim",
	event = { "BufReadPre", "BufNewFile" },
	opts = {
		formatters_by_ft = {
			-- Shell scripts
			fish = { "fish_indent" },
			sh = { "shfmt" },
			bash = { "shfmt" },

			-- Config files
			toml = { "taplo" },
			yaml = { "prettier" },
			json = { "prettier" },

			-- Documentation
			markdown = { "prettier" },

			-- Web/JS
			javascript = { "prettier" },
			typescript = { "prettier" },
			html = { "prettier" },
			css = { "prettier" },

			-- Lua (for Neovim configs)
			lua = { "stylua" },

			-- Python (only if black is installed)
			python = is_executable("black") and { "black" } or nil,
		},
	},

	keys = {
		{
			"<leader>cf",
			function()
				require("conform").format({ async = true })
			end,
			mode = "n",
			desc = "Format buffer",
		},
	},
}
