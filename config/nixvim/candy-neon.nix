# candy-neon.nix -- the Faelight Forest palette as a nvim colorscheme, in pure Nix.
# (INT-090 x INT-091.) Disables gruvbox; hand-defines highlight groups in the forest palette.
{
  highlight = {
    Normal       = { fg = "#C8E6B0"; bg = "#0B130B"; };
    Comment      = { fg = "#556655"; italic = true; };
    Keyword      = { fg = "#A6E22E"; bold = true; };
    Statement    = { fg = "#A6E22E"; };
    Function     = { fg = "#36E0D0"; };
    Type         = { fg = "#36E0D0"; bold = true; };
    String       = { fg = "#FF5C57"; };
    Number       = { fg = "#FF8F8B"; };
    Constant     = { fg = "#8CC26B"; };
    Identifier   = { fg = "#C8E6B0"; };
    Operator     = { fg = "#36E0D0"; };
    CursorLineNr = { fg = "#A6E22E"; bold = true; };
    LineNr       = { fg = "#3A4A3A"; };
    Visual       = { bg = "#2A3A2A"; };
    Search       = { fg = "#0B130B"; bg = "#A6E22E"; };
    Pmenu        = { fg = "#C8E6B0"; bg = "#132013"; };
    PmenuSel     = { fg = "#0B130B"; bg = "#36E0D0"; };
    StatusLine   = { fg = "#A6E22E"; bg = "#132013"; };
  };

  opts.cursorline = true;
}
