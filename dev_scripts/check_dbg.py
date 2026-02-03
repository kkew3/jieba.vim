# Check if any rust paths specified at argv contains "dbg!", "print!",
# "println!", "eprint!", "eprintln!". And if any python paths contains
# "pdb.set_trace()" Return 1 if it's indeed the case.
#
# Note that this script relies on `grep` to work.

import subprocess
import sys
import os


def assert_not_found_in(path: str, fixed_patterns: list[str]):
    if not fixed_patterns:
        return

    cmd = ["grep", "-w", "-F"]
    for fp in fixed_patterns:
        cmd.append("-e")
        cmd.append(fp)
    cmd.append(path)
    try:
        proc = subprocess.run(cmd, check=True)
        if proc.returncode == 0:
            raise ValueError(f"debugging lines found in: {path}")
    except subprocess.CalledProcessError as err:
        if err.returncode > 1:
            raise
        if err.returncode == 1:
            pass


patterns = {
    ".py": ["pdb.set_trace()"],
    ".rs": ["dbg!"],
}

for path in sys.argv[1:]:
    assert_not_found_in(path, patterns.get(os.path.splitext(path)[1], []))
