# forest-nvim -- nixCats build of the Faelight Forest neovim (INT-122).
# Migrated from nixvim (nix/home/dotfiles/nixvim/). nixCats splits cleanly:
#   - THIS FILE (Nix): which plugins + tools to PACKAGE (categoryDefinitions)
#   - plugin/*.lua + init.lua (real Lua): how they are CONFIGURED
# Faithful port -- exact parity with the prior nixvim plugin set, in real Lua.
{ pkgs, nixCats, ... }:
let
  utils = import nixCats;
  luaPath = ./.;

  categoryDefinitions = { pkgs, ... }: {
    # CLI tools / LSP servers on the nvim runtime PATH
    lspsAndRuntimeDeps = {
      general = with pkgs; [
        ripgrep      # telescope live_grep backend
        fd           # telescope find_files backend
      ];
    };
    # loaded at startup
    startupPlugins = {
      general = with pkgs.vimPlugins; [
        # deps
        plenary-nvim
        nvim-web-devicons
        nui-nvim
        # UI / statusline / tabs
        lualine-nvim
        bufferline-nvim
        which-key-nvim
        # syntax
        nvim-treesitter.withAllGrammars
        # finder + tree
        telescope-nvim
        neo-tree-nvim
        # git + editing
        gitsigns-nvim
        comment-nvim
        # completion (nvim-cmp stack, faithful to nixvim)
        nvim-cmp
        luasnip
        cmp_luasnip
        cmp-nvim-lsp
        cmp-buffer
        cmp-path
      ];
    };
    optionalPlugins = { general = with pkgs.vimPlugins; [ ]; };
  };

  packageDefinitions = {
    forest-nvim = { pkgs, ... }: {
      settings = {
        suffix-path = true;
        suffix-LD = true;
        hosts.python3.enable = false;
        hosts.node.enable = false;
        hosts.ruby.enable = false;
        hosts.perl.enable = false;
      };
      categories = { general = true; };
      extra = {};
    };
  };

  defaultPackageName = "forest-nvim";
in
  utils.baseBuilder luaPath { inherit pkgs; } categoryDefinitions packageDefinitions defaultPackageName
