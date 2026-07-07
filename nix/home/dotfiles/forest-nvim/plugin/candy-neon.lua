-- candy-neon.lua -- Faelight Forest palette, real Lua (INT-122 port of candy-neon.nix).
-- Direct 1:1 port of the nixvim highlight-group attrset -> actual nvim_set_hl calls.
vim.opt.cursorline = true
local hl = vim.api.nvim_set_hl
hl(0, "Normal",       { fg = "#C8E6B0", bg = "#0B130B" })
hl(0, "Comment",      { fg = "#556655", italic = true })
hl(0, "Keyword",      { fg = "#A6E22E", bold = true })
hl(0, "Statement",    { fg = "#A6E22E" })
hl(0, "Function",     { fg = "#36E0D0" })
hl(0, "Type",         { fg = "#36E0D0", bold = true })
hl(0, "String",       { fg = "#FF5C57" })
hl(0, "Number",       { fg = "#FF8F8B" })
hl(0, "Constant",     { fg = "#8CC26B" })
hl(0, "Identifier",   { fg = "#C8E6B0" })
hl(0, "Operator",     { fg = "#36E0D0" })
hl(0, "CursorLineNr", { fg = "#A6E22E", bold = true })
hl(0, "LineNr",       { fg = "#3A4A3A" })
hl(0, "Visual",       { bg = "#2A3A2A" })
hl(0, "Search",       { fg = "#0B130B", bg = "#A6E22E" })
hl(0, "Pmenu",        { fg = "#C8E6B0", bg = "#132013" })
hl(0, "PmenuSel",     { fg = "#0B130B", bg = "#36E0D0" })
hl(0, "StatusLine",   { fg = "#A6E22E", bg = "#132013" })
