-- LSP Configuration for Faelight Forest
-- Optimized for Lua configs and shell scripts

return {
	{
		"neovim/nvim-lspconfig",
		opts = {
			servers = {
				-- Lua LSP settings
				lua_ls = {
					settings = {
						Lua = {
							workspace = {
								checkThirdParty = false,
							},
							completion = {
								callSnippet = "Replace",
							},
							diagnostics = {
								globals = { "vim" },
							},
						},
					},
				},

				-- Bash LSP
				bashls = {},

				-- JSON LSP
				jsonls = {},

				-- Markdown LSP
				marksman = {},
			},
		},
	},
}
