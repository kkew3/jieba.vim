# Changelog

## v2.1.1 - 2026-06-03

Bug fixes:

- `cw` incorrectly resets the IM when working with [`im-select.nvim`][im-select] ([#83], [#99]).
- Yank does not work with [`vim-highlightedyank`][vim-highlight] or Neovim [`vim.hl`][vim.hl] ([#88], [#99]).
- `.` does not repeat last change after yanks or motion failure ([#105], [#106]).
- Yank incorrectly marks the buffer as modified ([#100], [#106]).
- Motion failure does not respect Vim's builtin behavior ([#64], [#107]).
- In xmap, `ge` at end-of-line incorrectly jumps to the start of selection ([#108], [#109]).

Documentation:

- Add FAQ ([#83], [`1583bfd`]).

Dev:

- Refactor metatest, reducing CI time 6x; implement integrated tests ([#102]).
- Simplify notations in jieba tests ([#104]).
- Bump core dependency [`jieba-rs`][jieba-rs] to 0.9 ([#103]).

[im-select]: https://github.com/keaising/im-select.nvim
[#83]: https://github.com/kkew3/jieba.vim/issues/83
[#99]: https://github.com/kkew3/jieba.vim/pull/99
[vim-highlight]: https://github.com/machakann/vim-highlightedyank
[vim.hl]: https://neovim.io/doc/user/lua/#vim.hl
[#88]: https://github.com/kkew3/jieba.vim/issues/88
[#105]: https://github.com/kkew3/jieba.vim/issues/105
[#106]: https://github.com/kkew3/jieba.vim/pull/106
[#100]: https://github.com/kkew3/jieba.vim/issues/100
[#64]: https://github.com/kkew3/jieba.vim/issues/64
[#107]: https://github.com/kkew3/jieba.vim/pull/107
[#108]: https://github.com/kkew3/jieba.vim/issues/108
[#109]: https://github.com/kkew3/jieba.vim/pull/109
[`1583bfd`]: https://github.com/kkew3/jieba.vim/commit/1583bfdf4568a7e82e03c6fe82f5c31f23d0ba54
[#102]: https://github.com/kkew3/jieba.vim/pull/102
[#104]: https://github.com/kkew3/jieba.vim/pull/104
[jieba-rs]: https://github.com/messense/jieba-rs
[#103]: https://github.com/kkew3/jieba.vim/pull/103


## v2.1.0 - 2026-03-27

Features:

- Add support to word text objects ([#32], [#85]).
- Don't need to restart vim after calling `jieba_vim#install()` for it to take effect ([#87], [#93]).

Bug fixes:

- Count not working under `xmap` and `omap` ([#86], [#90]).
- Spurious message during installation ([#91], [#92])

Documentation:

- Update README and Roadmap ([`11d11b6`]).

[#32]: https://github.com/kkew3/jieba.vim/issues/32
[#85]: https://github.com/kkew3/jieba.vim/pull/85
[#87]: https://github.com/kkew3/jieba.vim/issues/87
[#93]: https://github.com/kkew3/jieba.vim/pull/93
[#86]: https://github.com/kkew3/jieba.vim/issues/86
[#90]: https://github.com/kkew3/jieba.vim/pull/90
[#91]: https://github.com/kkew3/jieba.vim/issues/91
[#92]: https://github.com/kkew3/jieba.vim/pull/92
[`11d11b6`]: https://github.com/kkew3/jieba.vim/commit/11d11b61f0d04b14461b55a1917a4e31c0b4651f


## v2.0.0 - 2026-03-22

Features:

- Export lua binding ([#79]). Neovim users can use [jieba.vim] **out-of-the-box** (see README). No more `+python3` prerequisite!
- Cross platform build and install of the plugin through a uniform interface `jieba_vim#install()`.
- Cdylib for Vim and Neovim are now hosted under names `*-py3.*` and `*-lua51.*` respectively in the [Release] page.

Dev:

- Brand new testing framework `jieba_vim_rs_metatest` that facilitates over 30K tests against a running Vim oracle, ensuring absolute compliance to Vim's builtin behavior on ASCII text.
- Drop dev dependency on [vader.vim]; implement tests using `jieba_vim_rs_metatest` framework instead.
- Rewrite existing motions under Markovian motion framework, combining with low level api design inherited from Bram's original C codebase.
- Migrate complexity in `pythonx` to vimscript implementation, resulting in cleaner and more maintainable vim-side code.
- Several fixes in build scripts ([#74], [#76], [#77], [#81]).
- Several fixes to old bugs ([#15]).
- Start following [SemVer] in versioning.

Licensing:

- Several files (e.g. `rust_backend/jieba_vim_rs_core/src/motion/primitives/text_object/*.rs`) are now licensed under Vim license, since they are developed with reference to Bram's Vim source code.

Thanks:

- [@Bob-Eric] ([#74])
- [@TonyWu20] ([#77])

[jieba.vim]: https://github.com/kkew3/jieba.vim
[Release]: https://github.com/kkew3/jieba.vim/releases
[vader.vim]: https://github.com/junegunn/vader.vim
[#79]: https://github.com/kkew3/jieba.vim/pull/79
[#74]: https://github.com/kkew3/jieba.vim/pull/74
[#76]: https://github.com/kkew3/jieba.vim/pull/76
[#77]: https://github.com/kkew3/jieba.vim/pull/77
[#81]: https://github.com/kkew3/jieba.vim/pull/81
[SemVer]: https://semver.org
[#15]: https://github.com/kkew3/jieba.vim/issues/15
[@Bob-Eric]: https://github.com/Bob-Eric
[@TonyWu20]: https://github.com/TonyWu20


## v1.0.6 - 2025-12-28 (YANKED)

Dev:

- Upgrade rust and dependencies ([#38], [#44], [#45]).
- Fix lint and formatting ([#46], [#47]).
- Add intel macOS build to CI/CD ([#48]).

### NOTICE

This release accidentally breaks the pre-built cdylib downloading function in the vim post-install script (#50). If you insist on installing this version, nothing should misbehave except that you will need to compile rust from source locally. We will fix this issue in the next release. Right now, we suggest sticking to `v1.0.5`, and only if you're comfortable with compiling from source, upgrade to `v1.0.6`.

[#38]: https://github.com/kkew3/jieba.vim/pull/38
[#44]: https://github.com/kkew3/jieba.vim/pull/44
[#45]: https://github.com/kkew3/jieba.vim/pull/45
[#46]: https://github.com/kkew3/jieba.vim/pull/46
[#47]: https://github.com/kkew3/jieba.vim/pull/47
[#48]: https://github.com/kkew3/jieba.vim/pull/48


## v1.0.5 - 2025-02-16

Features:

- Add support to `'iskeyword'` Vim option ([#16], [#20]).

Dev:

- Improve documentation ([#18]).
- Enable support of aarch64-unknown-linux-gnu platform ([#21]).
- Move rust backend to its own directory ([#19], [#22]).
- Improve internal data structure ([#26], [#27]).

Thanks:

- [@pu-007] ([#18])

[#16]: https://github.com/kkew3/jieba.vim/issues/16
[#20]: https://github.com/kkew3/jieba.vim/pull/20
[#18]: https://github.com/kkew3/jieba.vim/pull/18
[#21]: https://github.com/kkew3/jieba.vim/pull/21
[#19]: https://github.com/kkew3/jieba.vim/issues/19
[#22]: https://github.com/kkew3/jieba.vim/pull/22
[#26]: https://github.com/kkew3/jieba.vim/issues/26
[#27]: https://github.com/kkew3/jieba.vim/pull/27
[@pu-007]: https://github.com/pu-007


## v1.0.4 - 2025-01-13

Features:

- Provide precompiled shared library for major OS and platforms so that local Rust environment is not needed ([#10], [`189a086`]).
- Dot-repeat of word operations based on [`tpope/vim-repeat`][vim-repeat] ([#5], [#6], [#14]).

Bug fixes:

- Incomplete register ([#11], [#13]).
- Crash in tokenization ([#14]).
- Cursor getting stuck in visual mode ([#17]).

Dev:

- Improve integration tests ([#12]).

[#10]: https://github.com/kkew3/jieba.vim/issues/10
[`189a086`]: https://github.com/kkew3/jieba.vim/commit/189a086b9e1a67b269d74f4a5ebdec27441237a1
[vim-repeat]: https://github.com/tpope/vim-repeat.git
[#5]: https://github.com/kkew3/jieba.vim/issues/5
[#6]: https://github.com/kkew3/jieba.vim/issues/6
[#14]: https://github.com/kkew3/jieba.vim/pull/14
[#11]: https://github.com/kkew3/jieba.vim/issues/11
[#13]: https://github.com/kkew3/jieba.vim/pull/13
[#14]: https://github.com/kkew3/jieba.vim/pull/14
[#17]: https://github.com/kkew3/jieba.vim/pull/17
[#12]: https://github.com/kkew3/jieba.vim/pull/12


## v1.0.3 - 2024-12-30

Features:

- Add docs. Use `:h jieba` in Vim to check for docs. You may need to first generate help tags by `:helptags`.
- Add support to spacing modifier letters, combining diacritical marks, and emoji in tokenization.
- **Breaking change**: Simplify Chinese punctuation association rule. This involves keeping only a number of essential right-punctuation like Chinese comma and full-stop, and removing all others. The goal is to be more consistent with Vim's default behavior. Only WORD motions are affected.

Thanks:

- [@wsdjeg] ([#4])

[@wsdjeg]: https://github.com/wsdjeg
[#4]: https://github.com/kkew3/jieba.vim/pull/4
