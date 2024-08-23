#!/bin/bash
set -e

echo "=== Downloading the word list from cutword ==="
mkdir -p pythonx/src/data
curl -o pythonx/src/data/unionwords.txt "https://raw.githubusercontent.com/liwenju0/cutword/main/cutword/unionwords.txt"
echo "=== Building rust ==="
cd pythonx && cargo build -r
