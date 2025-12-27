We recommend [`astral-sh/uv`](https://github.com/astral-sh/uv) for managing the test environment.

To install dependencies for integration tests, simply:

```bash
uv sync
```

To run tests, either ensure "vim"/"nvim" is in `PATH`, or specify the absolute path to them in `VIM_BIN_NAME`:

```bash
VIM_BIN_NAME=vim uv run pytest
VIM_BIN_NAME=nvim uv run pytest
```
