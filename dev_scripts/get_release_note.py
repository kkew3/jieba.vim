# Get release note from CHANGELOG.md.
#
# Usage: uv run --project dev_scripts dev_scripts/get_release_note.py vX.X.X
# To get the first section:
#        uv run --project dev_scripts dev_scripts/get_release_note.py

import re
import sys


def get_release_note(tag_name: str) -> list[str]:
    assert tag_name, "tag_name must not be empty"
    release_note_lines = []
    in_release_note = False
    with open("CHANGELOG.md", encoding="utf-8") as infile:
        for line in infile:
            line = line.rstrip("\n")
            if line.startswith("## "):
                in_release_note = line.startswith(f"## {tag_name}")
            if in_release_note:
                release_note_lines.append(line)
    drop_heading_and_surrounding_blank_lines_(release_note_lines)
    return release_note_lines


def get_release_note1() -> list[str]:
    """Get the first section in CHANGLELOG.md."""
    release_note_lines = []
    in_release_note = False
    with open("CHANGELOG.md", encoding="utf-8") as infile:
        for line in infile:
            line = line.rstrip("\n")
            if line.startswith("## "):
                if not in_release_note:
                    in_release_note = True
                else:
                    break
            if in_release_note:
                release_note_lines.append(line)
    drop_heading_and_surrounding_blank_lines_(release_note_lines)
    return release_note_lines


def drop_heading_and_surrounding_blank_lines_(release_note_lines: list[str]):
    # Pop trailing blank lines.
    while release_note_lines and not release_note_lines[-1].strip():
        release_note_lines.pop()
    # Drop the leading heading, since GitHub Releases page already shows the
    # release tag name and the released date.
    assert release_note_lines[0].startswith("## ")
    del release_note_lines[0]
    while release_note_lines and not release_note_lines[0].strip():
        del release_note_lines[0]


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
    tag_name = sys.argv[1] if sys.argv[1:] else None
    if tag_name is not None:
        release_note_lines = get_release_note(tag_name)
    else:
        release_note_lines = get_release_note1()
    fix_missing_links_(release_note_lines)
    print("\n".join(release_note_lines))


if __name__ == "__main__":
    main()
