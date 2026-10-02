# Vim 至少需要什么版本

本插件在 Vim-8.2.3455、Vim-9.1.0000、Neovim-0.10.2、Neovim-0.11.5、Neovim-0.12.2 上测试通过。所以至少需要 Vim-8.2.3455 或 Neovim-0.10.2。

# 如何与 im-select.nvim 配套使用

[`im-select.nvim`][im-select] 可以用于自动切换输入法。目前需要在 `set_default_events` 中禁用 `CmdlineLeave` 或设置 `async_switch_im = false`（见 issue [#83][issue83] 中的讨论）。一个在 macOS 下可行的 [lazy.nvim][lazy] 配置示例如下：

```lua
{
    "keaising/im-select.nvim",
    config = function()
        require('im_select').setup({
            default_command = { "/usr/local/bin/macism" },
            default_im_select  = "com.apple.keylayout.US",
            set_default_events = { "InsertLeave", "CmdlineLeave" },
            set_previous_events = { "InsertEnter" },
            async_switch_im = false
        })
    end,
}
```

# 如何与 vim-surround / nvim-surround 配套使用

[`vim-surround`][vim-surround] 和 [`nvim-surround`][nvim-surround] 可与 jieba.vim 搭配使用，例如为中文词语加括号。为此需要启用 `g:jieba_vim_experimental_opfunc`（[#150]）。可在 `~/.vimrc` 中做如下配置：

```vim
let g:jieba_vim_experimental_opfunc = 1
```

注意，我们在这里无法提供 100% 兼容 Vim 原生行为的保证，其局限性请参见 [#153] 与设计文档 [docs/design_choices/0003-omap-impl-tradeoff.md][0003]。

# 如何与 rime.vim 配套使用

[`rime.vim`][rime.vim] 是基于 [Rime][rime] 的 Vim/Neovim 中文输入法。为达成最大集成度同样可以考虑启用 `g:jieba_vim_experimental_opfunc` 开关。但这并不是必需的；如果介意其局限性且仅需基础集成，可以不启用。

# 是否支持加载 Rime 词典

见 [#156]。

[im-select]: https://github.com/keaising/im-select.nvim
[lazy]: https://lazy.folke.io/
[issue83]: https://github.com/kkew3/jieba.vim/issues/83
[vim-surround]: https://github.com/tpope/vim-surround
[nvim-surround]: https://github.com/kylechui/nvim-surround
[#150]: https://github.com/kkew3/jieba.vim/pull/150
[#153]: https://github.com/kkew3/jieba.vim/pull/153
[0003]: https://github.com/kkew3/jieba.vim/blob/main/docs/design_choices/0003-omap-impl-tradeoff.md
[rime.vim]: https://github.com/TSalmon3/rime.vim
[rime]: https://rime.im
[#156]: https://github.com/kkew3/jieba.vim/issues/156
