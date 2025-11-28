local map = vim.keymap.set

-- Better save
map({ "i", "n" }, "<C-s>", "<cmd>w<cr><esc>", { desc = "Save file" })

-- Clear search
map("n", "<Esc>", "<cmd>noh<cr>", { desc = "Clear search" })

-- Better indenting
map("v", "<", "<gv")
map("v", ">", ">gv")

-- Move lines
map("n", "<A-j>", ":m .+1<cr>==", { desc = "Move line down" })
map("n", "<A-k>", ":m .-2<cr>==", { desc = "Move line up" })

-- Better paste
map("v", "p", '"_dP', { desc = "Paste without losing clipboard" })

-- File navigation
map("n", "<leader>e", "<cmd>Neotree toggle<cr>", { desc = "Toggle explorer" })
map("n", "<leader>o", "<cmd>Neotree focus<cr>", { desc = "Focus explorer" })

-- Find files
map("n", "<leader>ff", "<cmd>Telescope find_files hidden=true<cr>", { desc = "Find files" })
map("n", "<leader>fg", "<cmd>Telescope live_grep<cr>", { desc = "Live grep" })
map("n", "<leader>fb", "<cmd>Telescope buffers<cr>", { desc = "Find buffers" })
map("n", "<leader>fr", "<cmd>Telescope oldfiles<cr>", { desc = "Recent files" })

-- Window navigation
map("n", "<C-h>", "<C-w>h", { desc = "Left window" })
map("n", "<C-j>", "<C-w>j", { desc = "Down window" })
map("n", "<C-k>", "<C-w>k", { desc = "Up window" })
map("n", "<C-l>", "<C-w>l", { desc = "Right window" })

-- Splits
map("n", "<leader>wv", "<cmd>vsplit<cr>", { desc = "Vertical split" })
map("n", "<leader>ws", "<cmd>split<cr>", { desc = "Horizontal split" })

-- Buffers
map("n", "<S-h>", "<cmd>bprevious<cr>", { desc = "Prev buffer" })
map("n", "<S-l>", "<cmd>bnext<cr>", { desc = "Next buffer" })
map("n", "<leader>bd", "<cmd>bdelete<cr>", { desc = "Delete buffer" })

-- Terminal
map("n", "<C-`>", "<cmd>ToggleTerm<cr>", { desc = "Toggle terminal" })
map("t", "<Esc><Esc>", "<C-\\><C-n>", { desc = "Exit terminal mode" })

-- LSP
map("n", "gd", vim.lsp.buf.definition, { desc = "Go to definition" })
map("n", "gr", vim.lsp.buf.references, { desc = "References" })
map("n", "K", vim.lsp.buf.hover, { desc = "Hover" })
map("n", "<leader>ca", vim.lsp.buf.code_action, { desc = "Code action" })
map("n", "<leader>cr", vim.lsp.buf.rename, { desc = "Rename" })
map("n", "<leader>cf", function()
  vim.lsp.buf.format({ async = true })
end, { desc = "Format" })

-- Diagnostics
map("n", "[d", vim.diagnostic.goto_prev, { desc = "Prev diagnostic" })
map("n", "]d", vim.diagnostic.goto_next, { desc = "Next diagnostic" })

-- Quick actions
map("n", ";", ":", { desc = "Enter command mode" })
map("n", "<leader>ul", "<cmd>set nu!<cr>", { desc = "Toggle line numbers" })
