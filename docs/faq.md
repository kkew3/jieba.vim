# Vim 至少需要什么版本

本插件在 Vim-8.2.3455、Vim-9.1.0000、Neovim-0.10.2、Neovim-0.11.5 上测试通过。所以至少需要 Vim-8.2.3455。

# 如何与 im-select.nvim 配套使用

[`im-select.nvim`][im-select] 可以用于自动切换输入法。目前需要在 `set_default_events` 中禁用 `CmdlineLeave`（见 issue [#83][issue83] 中的讨论）。一个在 macOS 下可行的 [lazy.nvim][lazy] 配置示例如下：

```lua
{
    "keaising/im-select.nvim",
    config = function()
        require('im_select').setup({
            default_command = { "/usr/local/bin/macism" },
            default_im_select  = "com.apple.keylayout.US",

            -- 此处不能有 "CmdlineLeave"
            set_default_events = { "InsertLeave" },

            set_previous_events = { "InsertEnter" },
            async_switch_im = true
        })
    end,
}
```


[im-select]: https://github.com/keaising/im-select.nvim
[lazy]: https://lazy.folke.io/
[issue83]: https://github.com/kkew3/jieba.vim/issues/83
