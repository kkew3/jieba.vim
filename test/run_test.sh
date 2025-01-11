#!/bin/bash
source venv/bin/activate
echo "=== vim integration tests ==="
VIM_BIN_NAME=vim pytest
echo
echo "=== nvim integration tests ==="
VIM_BIN_NAME=nvim pytest
