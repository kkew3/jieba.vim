-- Bootstrap lazy.nvim
local lazypath = vim.fn.stdpath("data") .. "/lazy/lazy.nvim"
if not (vim.uv or vim.loop).fs_stat(lazypath) then
  local lazyrepo = "https://github.com/folke/lazy.nvim.git"
  local out = vim.fn.system({ "git", "clone", "--filter=blob:none", "--branch=stable", lazyrepo, lazypath })
  if vim.v.shell_error ~= 0 then
    vim.api.nvim_echo({
      { "Failed to clone lazy.nvim:\n", "ErrorMsg" },
      { out, "WarningMsg" },
      { "\nPress any key to exit..." },
    }, true, {})
    vim.fn.getchar()
    os.exit(1)
  end
end
vim.opt.rtp:prepend(lazypath)

local script_dir = vim.fn.expand("<sfile>:p:h")

-- Necessary to install jieba.vim.
vim.o.compatible = false

-- Used to wrap up the benchmark session cleanly.
vim.o.swapfile = false

-- Setup lazy.nvim
require("lazy").setup({
  root = vim.fn.expand(script_dir .. "/.nvim_bundle"),
  spec = {
    {
      'kkew3/jieba.vim',
      branch = "release",
      build = ":call jieba_vim#install()",
      init = function()
        -- One-liner config for jieba.vim.
	      vim.g.jieba_vim_keymap = 1
      end,
    },
  },
})
