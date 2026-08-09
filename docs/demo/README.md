# Demo

Vim:

![vim demo](./demo_vim.gif)

Neovim:

![nvim demo](./demo_nvim.gif)

<p style="text-align: right;">—— powered by <a href="https://github.com/pkazmier/vhs/tree/caption-overlay">pkazmier/vhs</a></p>

## How to reproduce

1. Install [Go lang](https://go.dev); then install [pkazmier/vhs](https://github.com/pkazmier/vhs.git) from source.
2. Prepare Vim and Neovim; install [vim-plug](https://github.com/junegunn/vim-plug) and [lazy.nvim](https://lazy.folke.io), as well as the necessary plugins defined in [vimrc](./vimrc) and [init.lua](./init.lua) following their instructions.
3. Cd to current directory, and run `make clean all` to reproduce the GIFs. Alternatively, inspect [Makefile](./Makefile) and run individual commands yourself. Note that while admitting perceptual parity with the demo GIFs, the generated GIFs on your machine are usually *not* byte-to-byte identical to the demos.

## Help on the `*.tape` file

Check the main reference at <https://github.com/charmbracelet/vhs#vhs-command-reference> and the extended syntax at <https://github.com/charmbracelet/vhs/pull/719>.
