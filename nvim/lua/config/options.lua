-- Options are automatically loaded before lazy.nvim startup
-- Default options that are always set: https://github.com/LazyVim/LazyVim/blob/main/lua/lazyvim/config/options.lua
-- Add any additional options here
vim.opt.relativenumber = false

-- === Custom Productivity Options ===
local opt = vim.opt

-- UI
opt.cursorline = true
opt.scrolloff = 8
opt.signcolumn = "yes"

-- Editing
opt.expandtab = true
opt.shiftwidth = 2
opt.tabstop = 2
opt.smartindent = true
opt.wrap = false

-- Search
opt.ignorecase = true
opt.smartcase = true

-- Files
opt.hidden = true
opt.autowrite = true
opt.backup = false
opt.swapfile = false
opt.undofile = true

-- Clipboard & Mouse
opt.clipboard = "unnamedplus"
opt.mouse = "a"

-- Performance
opt.updatetime = 200
opt.timeoutlen = 300

-- Splits
opt.splitright = true
opt.splitbelow = true

-- Show invisible chars
opt.list = true
opt.listchars = { tab = "→ ", trail = "·", nbsp = "␣" }

-- Folds
opt.foldmethod = "expr"
opt.foldexpr = "nvim_treesitter#foldexpr()"
opt.foldlevel = 99

-- Use ripgrep
if vim.fn.executable("rg") == 1 then
  opt.grepprg = "rg --vimgrep --smart-case --hidden"
end

-- Fish shell fix
if vim.fn.executable("bash") == 1 then
  opt.shell = "/bin/bash"
end

-- Highlight yank
vim.api.nvim_create_autocmd("TextYankPost", {
  callback = function()
    vim.highlight.on_yank({ timeout = 200 })
  end,
})
