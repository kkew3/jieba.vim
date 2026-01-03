# Run the vader test provided at argv manually with vim.

import argparse
from pathlib import Path
import subprocess
import sys


def search_vimrc() -> Path | None:
    curdir = Path.cwd()
    project_root = Path(
        subprocess.check_output(
            ["git", "rev-parse", "--show-toplevel"], text=True
        ).strip()
    )
    assert curdir.is_relative_to(project_root), (
        "curdir must be below jieba.vim project root"
    )
    while curdir != project_root:
        vimrc_file = curdir / "vimrc"
        if vimrc_file.is_file():
            return vimrc_file
        curdir = curdir.parent
    return None


parser = argparse.ArgumentParser(description="Run vader test.")
parser.add_argument("-b", "--bang", action="store_true", help="run with bang")
parser.add_argument(
    "-c", "--vimrc", type=Path, default=search_vimrc(), help="the vimrc path"
)
parser.add_argument("vader_file", type=Path, help="the vader test file")
args = parser.parse_args()

vader_file = args.vader_file
if not vader_file.endswith(".vader"):
    vader_file += ".vader"
assert " " not in vader_file, "vader_file path must not contain space"
assert args.vimrc is not None, "vimrc path not specified"
if not args.bang:
    proc = subprocess.run(["vim", "-Nu", args.vimrc, f"+:Vader {vader_file}"])
else:
    proc = subprocess.run(["vim", "-Nu", args.vimrc, f"+:Vader! {vader_file}"])
sys.exit(proc.returncode)
