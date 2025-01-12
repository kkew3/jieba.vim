To install dependencies for integration tests, simply:

```bash
python3 -m venv venv
pip install -e .
```

To run tests, either ensure "vim"/"nvim" is in `PATH`, or specify the absolute path to them in `VIM_BIN_NAME`:

```bash
source venv/bin/activate
VIM_BIN_NAME=vim pytest
VIM_BIN_NAME=nvim pytest
```
