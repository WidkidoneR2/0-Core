-- Faelight Forest Theme for Neovim
vim.cmd("hi clear")
if vim.fn.exists("syntax_on") then
  vim.cmd("syntax reset")
end
vim.o.termguicolors = true
vim.g.colors_name = "faelight"

local c = {
  bg = "#0f1411",
  bg_light = "#1a1f1c",
  bg_lighter = "#252b27",
  fg = "#d7e0da",
  fg_dim = "#778f7f",
  green = "#6be3a3",
  blue = "#5cc8ff",
  amber = "#f5c177",
  red = "#e67e80",
  purple = "#d699b6",
  cyan = "#7fc8c8",
}

local hi = function(group, opts)
  vim.api.nvim_set_hl(0, group, opts)
end

-- Editor
hi("Normal", { fg = c.fg, bg = c.bg })
hi("NormalFloat", { fg = c.fg, bg = c.bg_light })
hi("FloatBorder", { fg = c.green, bg = c.bg_light })
hi("Cursor", { fg = c.bg, bg = c.green })
hi("CursorLine", { bg = c.bg_light })
hi("CursorLineNr", { fg = c.green, bold = true })
hi("LineNr", { fg = c.fg_dim })
hi("Visual", { bg = c.bg_lighter })
hi("Search", { fg = c.bg, bg = c.amber })
hi("IncSearch", { fg = c.bg, bg = c.green })
hi("StatusLine", { fg = c.fg, bg = c.bg_light })
hi("StatusLineNC", { fg = c.fg_dim, bg = c.bg_light })
hi("VertSplit", { fg = c.bg_lighter })
hi("WinSeparator", { fg = c.bg_lighter })
hi("Pmenu", { fg = c.fg, bg = c.bg_light })
hi("PmenuSel", { fg = c.bg, bg = c.green })
hi("PmenuSbar", { bg = c.bg_lighter })
hi("PmenuThumb", { bg = c.green })
hi("TabLine", { fg = c.fg_dim, bg = c.bg_light })
hi("TabLineFill", { bg = c.bg_light })
hi("TabLineSel", { fg = c.green, bg = c.bg })
hi("SignColumn", { fg = c.fg_dim, bg = c.bg })
hi("ModeMsg", { fg = c.green, bold = true })
hi("ErrorMsg", { fg = c.red })
hi("WarningMsg", { fg = c.amber })
hi("Directory", { fg = c.blue })
hi("Title", { fg = c.green, bold = true })
hi("NonText", { fg = c.bg_lighter })
hi("Folded", { fg = c.fg_dim, bg = c.bg_light })
hi("MatchParen", { fg = c.amber, bold = true })
hi("ColorColumn", { bg = c.bg_light })

-- Syntax
hi("Comment", { fg = c.fg_dim, italic = true })
hi("Constant", { fg = c.amber })
hi("String", { fg = c.green })
hi("Character", { fg = c.green })
hi("Number", { fg = c.amber })
hi("Boolean", { fg = c.amber })
hi("Float", { fg = c.amber })
hi("Identifier", { fg = c.fg })
hi("Function", { fg = c.blue })
hi("Statement", { fg = c.purple })
hi("Conditional", { fg = c.purple })
hi("Repeat", { fg = c.purple })
hi("Label", { fg = c.purple })
hi("Operator", { fg = c.cyan })
hi("Keyword", { fg = c.purple })
hi("Exception", { fg = c.purple })
hi("PreProc", { fg = c.cyan })
hi("Include", { fg = c.cyan })
hi("Define", { fg = c.cyan })
hi("Macro", { fg = c.cyan })
hi("Type", { fg = c.blue })
hi("StorageClass", { fg = c.purple })
hi("Structure", { fg = c.blue })
hi("Typedef", { fg = c.blue })
hi("Special", { fg = c.cyan })
hi("Tag", { fg = c.green })
hi("Delimiter", { fg = c.fg })
hi("Error", { fg = c.red })
hi("Todo", { fg = c.bg, bg = c.amber, bold = true })

-- Treesitter
hi("@variable", { fg = c.fg })
hi("@variable.builtin", { fg = c.cyan })
hi("@constant", { fg = c.amber })
hi("@constant.builtin", { fg = c.amber })
hi("@module", { fg = c.blue })
hi("@string", { fg = c.green })
hi("@string.escape", { fg = c.cyan })
hi("@character", { fg = c.green })
hi("@number", { fg = c.amber })
hi("@boolean", { fg = c.amber })
hi("@function", { fg = c.blue })
hi("@function.builtin", { fg = c.blue })
hi("@function.call", { fg = c.blue })
hi("@function.macro", { fg = c.cyan })
hi("@method", { fg = c.blue })
hi("@method.call", { fg = c.blue })
hi("@constructor", { fg = c.blue })
hi("@keyword", { fg = c.purple })
hi("@keyword.function", { fg = c.purple })
hi("@keyword.operator", { fg = c.cyan })
hi("@keyword.return", { fg = c.purple })
hi("@conditional", { fg = c.purple })
hi("@repeat", { fg = c.purple })
hi("@operator", { fg = c.cyan })
hi("@type", { fg = c.blue })
hi("@type.builtin", { fg = c.blue })
hi("@property", { fg = c.fg })
hi("@punctuation.bracket", { fg = c.fg_dim })
hi("@punctuation.delimiter", { fg = c.fg_dim })
hi("@punctuation.special", { fg = c.cyan })
hi("@comment", { fg = c.fg_dim, italic = true })
hi("@tag", { fg = c.green })
hi("@tag.attribute", { fg = c.blue })
hi("@tag.delimiter", { fg = c.fg_dim })

-- LSP
hi("DiagnosticError", { fg = c.red })
hi("DiagnosticWarn", { fg = c.amber })
hi("DiagnosticInfo", { fg = c.blue })
hi("DiagnosticHint", { fg = c.green })
hi("DiagnosticUnderlineError", { undercurl = true, sp = c.red })
hi("DiagnosticUnderlineWarn", { undercurl = true, sp = c.amber })
hi("DiagnosticUnderlineInfo", { undercurl = true, sp = c.blue })
hi("DiagnosticUnderlineHint", { undercurl = true, sp = c.green })
hi("LspReferenceText", { bg = c.bg_lighter })
hi("LspReferenceRead", { bg = c.bg_lighter })
hi("LspReferenceWrite", { bg = c.bg_lighter })

-- Git
hi("GitSignsAdd", { fg = c.green })
hi("GitSignsChange", { fg = c.blue })
hi("GitSignsDelete", { fg = c.red })
hi("DiffAdd", { bg = "#1a2f1a" })
hi("DiffChange", { bg = "#1a1f2f" })
hi("DiffDelete", { bg = "#2f1a1a" })
hi("DiffText", { bg = "#2f2f1a" })

-- Telescope
hi("TelescopeBorder", { fg = c.green })
hi("TelescopePromptBorder", { fg = c.green })
hi("TelescopeResultsBorder", { fg = c.green })
hi("TelescopePreviewBorder", { fg = c.green })
hi("TelescopeSelection", { bg = c.bg_lighter })
hi("TelescopeMatching", { fg = c.amber })

-- Neo-tree
hi("NeoTreeNormal", { fg = c.fg, bg = c.bg })
hi("NeoTreeNormalNC", { fg = c.fg, bg = c.bg })
hi("NeoTreeDirectoryIcon", { fg = c.blue })
hi("NeoTreeDirectoryName", { fg = c.blue })
hi("NeoTreeGitAdded", { fg = c.green })
hi("NeoTreeGitModified", { fg = c.amber })
hi("NeoTreeGitDeleted", { fg = c.red })

-- Indent
hi("IblIndent", { fg = c.bg_lighter })
hi("IblScope", { fg = c.green })

-- Which-key
hi("WhichKey", { fg = c.green })
hi("WhichKeyGroup", { fg = c.blue })
hi("WhichKeyDesc", { fg = c.fg })

-- Notify
hi("NotifyERRORBorder", { fg = c.red })
hi("NotifyWARNBorder", { fg = c.amber })
hi("NotifyINFOBorder", { fg = c.green })
hi("NotifyERRORIcon", { fg = c.red })
hi("NotifyWARNIcon", { fg = c.amber })
hi("NotifyINFOIcon", { fg = c.green })
hi("NotifyERRORTitle", { fg = c.red })
hi("NotifyWARNTitle", { fg = c.amber })
hi("NotifyINFOTitle", { fg = c.green })

-- Cmp
hi("CmpItemAbbrMatch", { fg = c.green, bold = true })
hi("CmpItemAbbrMatchFuzzy", { fg = c.green })
hi("CmpItemKind", { fg = c.blue })
hi("CmpItemMenu", { fg = c.fg_dim })
