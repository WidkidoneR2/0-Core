-- Faelight Daemon Integration
local M = {}

local socket_path = "/tmp/faelight-daemon.sock"

-- Helper to send command to daemon
local function send_daemon_command(command_name, command_data)
  local payload = {
    type = "Command",
    [command_name] = command_data or vim.NIL
  }
  
  local message = {
    id = os.time(),
    payload = payload
  }
  
  local json = vim.fn.json_encode(message)
  local cmd = string.format("echo '%s' | socat - UNIX-CONNECT:%s", json, socket_path)
  local result = vim.fn.system(cmd)
  
  if vim.v.shell_error ~= 0 then
    vim.notify("Faelight daemon error", vim.log.levels.ERROR)
    return nil
  end
  
  return vim.fn.json_decode(result)
end

-- Command: Ping daemon
vim.api.nvim_create_user_command("FaelightPing", function()
  local response = send_daemon_command("Ping")
  
  if response and response.payload.Pong ~= nil then
    vim.notify("🌲 Faelight daemon is alive!", vim.log.levels.INFO)
  else
    vim.notify("❌ Daemon not responding", vim.log.levels.ERROR)
  end
end, {})

-- Command: List files
vim.api.nvim_create_user_command("FaelightOpen", function()
  local cwd = vim.fn.getcwd()
  local response = send_daemon_command("GetEntries", { path = cwd })
  
  if response and response.payload.Entries then
    local entries = response.payload.Entries.entries
    vim.notify(string.format("Found %d entries", #entries), vim.log.levels.INFO)
    
    for i, entry in ipairs(entries) do
      if i <= 10 then
        print(string.format("%s %s", entry.is_dir and "📁" or "📄", entry.name))
      end
    end
  end
end, {})

-- Keybindings
vim.keymap.set("n", "<leader>fP", ":FaelightPing<CR>", { desc = "Faelight: Ping daemon" })
vim.keymap.set("n", "<leader>fo", ":FaelightOpen<CR>", { desc = "Faelight: List files" })

vim.notify("🌲 Faelight commands loaded!", vim.log.levels.INFO)

return M
