-- init.lua -- Faelight Forest neovim base config (INT-122 nixCats port).
-- Faithful real-Lua translation of the old nixvim forest.nix opts/globals/keymaps.

-- GLOBALS -- leader must be set before plugins load.
vim.g.mapleader = " "
vim.g.maplocalleader = " "

-- OPTS -- direct port of forest.nix `opts`.
vim.opt.number = true          -- absolute number on cursor line
vim.opt.relativenumber = true  -- relative numbers elsewhere
vim.opt.shiftwidth = 2         -- 2-space indents
vim.opt.tabstop = 2
vim.opt.expandtab = true       -- spaces not tabs
vim.opt.cursorline = true      -- highlight current line
vim.opt.wrap = false
vim.opt.scrolloff = 8          -- keep 8 lines above/below cursor
vim.opt.termguicolors = true   -- 24-bit color (candy-neon needs this)

-- KEYMAPS -- direct port of forest.nix `keymaps`.
local map = vim.keymap.set
map("n", "<leader>w", "<cmd>w<cr>",            { desc = "Write file" })
map("n", "<leader>q", "<cmd>q<cr>",            { desc = "Quit window" })
map("n", "<leader>e", "<cmd>Neotree toggle<cr>", { desc = "Toggle file tree" })
