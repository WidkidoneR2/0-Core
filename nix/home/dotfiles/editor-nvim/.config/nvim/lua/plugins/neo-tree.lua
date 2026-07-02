-- Telescope Configuration - Enhanced File Finding
return {
  "nvim-telescope/telescope.nvim",
  dependencies = {
    "nvim-lua/plenary.nvim",
    {
      "nvim-telescope/telescope-fzf-native.nvim",
      build = "make",
    },
  },
  opts = {
    defaults = {
      prompt_prefix = "   ",
      selection_caret = " ",
      entry_prefix = "  ",
      layout_strategy = "horizontal",
      layout_config = {
        horizontal = {
          prompt_position = "top",
          preview_width = 0.55,
        },
        width = 0.87,
        height = 0.80,
      },
      sorting_strategy = "ascending",
      file_ignore_patterns = {
        "^.git/",
        "node_modules/",
      },
      mappings = {
        i = {
          ["<C-j>"] = "move_selection_next",
          ["<C-k>"] = "move_selection_previous",
        },
      },
    },
    pickers = {
      find_files = {
        hidden = true,
        find_command = {
          "rg",
          "--files",
          "--hidden",
          "--glob",
          "!**/.git/*",
        },
      },
      live_grep = {
        additional_args = function()
          return { "--hidden" }
        end,
      },
    },
  },
}
