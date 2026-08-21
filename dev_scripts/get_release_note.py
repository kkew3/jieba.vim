# Get release note from CHANGELOG.md.
#
# Usage: uv run --project dev_scripts dev_scripts/get_release_note.py vX.X.X

import re
import subprocess
import sys


def get_release_note(tag_name: str) -> list[str]:
    release_note_lines = subprocess.run(
        [
            "perl",
            "-sne",
            "if(/^## /){$p=0;$p=1 if /^## \\Q$v/} print if $p",
            "--",
            f"-v={tag_name}",
            "CHANGELOG.md",
        ],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.splitlines()
    # Pop trailing blank lines.
    while release_note_lines and not release_note_lines[-1].strip():
        release_note_lines.pop()
    return release_note_lines


def fix_missing_links_(release_note_lines: list[str]) -> None:
    link_pattern = re.compile(r"(\[[^]]+\]): \S+")
    global_link_def = {}
    with open("CHANGELOG.md", encoding="utf-8") as infile:
        for line in infile:
            matchobj = link_pattern.fullmatch(line.strip())
            if matchobj:
                global_link_def[matchobj.group(1)] = matchobj.group(0)
    local_link_def = {}
    for line in release_note_lines:
        matchobj = link_pattern.fullmatch(line)
        if matchobj:
            local_link_def[matchobj.group(1)] = matchobj.group(0)

    # Sanity check that shared links are indeed identically defined.
    shared_link_def = global_link_def.keys() & local_link_def.keys()
    for link in shared_link_def:
        assert global_link_def[link] == local_link_def[link]

    possibly_missing_link_def = global_link_def.keys() - local_link_def.keys()
    missing_link_def = set()
    for line in release_note_lines:
        for link in possibly_missing_link_def:
            if link in line:
                missing_link_def.add(global_link_def[link])
    release_note_lines.extend(sorted(missing_link_def))


def main():
    tag_name = sys.argv[1]
    release_note_lines = get_release_note(tag_name)
    fix_missing_links_(release_note_lines)
    print("\n".join(release_note_lines))


if __name__ == "__main__":
    main()
