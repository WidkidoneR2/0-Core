# forest.nix -- a first hand-written nixvim config slice (INT-090 Phase 2).
# Everything here is Nix module syntax. nixvim turns these attrs into neovim config.
{
  # OPTS -- vim options. In lua this is `vim.opt.number = true`; in nixvim it's just attrs.
  opts = {
    number = true;          # absolute line number on the cursor line
    relativenumber = true;  # relative numbers everywhere else (jump with 5j etc.)
    shiftwidth = 2;         # 2-space indents
    tabstop = 2;
    expandtab = true;       # spaces, not tabs
    cursorline = true;      # highlight the line you're on
    wrap = false;
    scrolloff = 8;          # keep 8 lines visible above/below the cursor
  };

  # GLOBALS -- e.g. set the leader key to space (vim.g.mapleader = " ").
  globals.mapleader = " ";

  # A PLUGIN -- lualine, a statusline. One line to enable; nixvim wires the rest.
  plugins.lualine = {
    enable = true;
    settings.options.theme = "auto";  # follows our candy-neon highlights
  };

  # A PLUGIN -- which-key, shows keybind hints in a popup. Great for learning.
  plugins.which-key.enable = true;

  # TREESITTER -- real syntax highlighting via parsed grammars (not regex).
  # nixvim builds the grammars for you; just enable it.
  plugins.treesitter = {
    enable = true;
    settings = {
      highlight.enable = true;
      indent.enable = true;
    };
  };

  # TELESCOPE -- fuzzy finder (files, live grep, buffers). The neovim UX crown jewel.
  # Keymaps below open it. <leader>ff = find files, <leader>fg = grep text.
  plugins.telescope = {
    enable = true;
    keymaps = {
      "<leader>ff" = "find_files";
      "<leader>fg" = "live_grep";
      "<leader>fb" = "buffers";
    };
  };

  # WEB-DEVICONS -- file-type icons (needed by neo-tree/telescope for pretty icons).
  plugins.web-devicons.enable = true;

  # NEO-TREE -- a file-tree sidebar. <leader>e toggles it (keymap below).
  plugins.neo-tree.enable = true;

  # GITSIGNS -- git diff markers in the sign column (added/changed/removed lines).
  plugins.gitsigns.enable = true;

  # COMMENT -- toggle comments: gcc (line), gc (visual selection). 'gc' in normal+visual.
  plugins.comment.enable = true;

  # AUTOCOMPLETE -- nvim-cmp + a snippet engine (luasnip) + sources. This shows how
  # nixvim composes several plugins into one feature, all declared as Nix attrs.
  plugins.luasnip.enable = true;          # snippet engine cmp uses
  plugins.cmp = {
    enable = true;
    autoEnableSources = true;             # nixvim wires the sources below automatically
    settings = {
      sources = [
        { name = "nvim_lsp"; }
        { name = "luasnip"; }
        { name = "buffer"; }
        { name = "path"; }
      ];
      mapping = {
        "<C-Space>" = "cmp.mapping.complete()";       # Ctrl-Space: force-open the menu
        "<CR>" = "cmp.mapping.confirm({ select = true })";  # Enter: accept
        "<Tab>" = "cmp.mapping.select_next_item()";    # Tab: next suggestion
        "<S-Tab>" = "cmp.mapping.select_prev_item()";  # Shift-Tab: previous
      };
    };
  };

  # A KEYMAP -- <leader>w writes the file. Shows nixvim's keymap list syntax.
  keymaps = [
    {
      mode = "n";
      key = "<leader>w";
      action = "<cmd>w<cr>";
      options.desc = "Write file";
    }
    {
      mode = "n";
      key = "<leader>q";
      action = "<cmd>q<cr>";
      options.desc = "Quit window";
    }
    {
      mode = "n";
      key = "<leader>e";
      action = "<cmd>Neotree toggle<cr>";
      options.desc = "Toggle file tree";
    }
  ];
}
