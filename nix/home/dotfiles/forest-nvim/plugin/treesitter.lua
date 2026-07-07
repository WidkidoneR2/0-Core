-- treesitter.lua -- port of nixvim plugins.treesitter (highlight).
-- Grammars provided by Nix (nvim-treesitter.withAllGrammars). Modern nvim-treesitter
-- removed `.configs.setup{}`; start highlighting per-buffer via vim.treesitter.start.
vim.api.nvim_create_autocmd('FileType', {
  callback = function(args)
    pcall(vim.treesitter.start, args.buf)
  end,
})
