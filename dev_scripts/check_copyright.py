# Check if the copyright notices of paths specified at argv are outdated. Exit
# with nonzero code if any copyright notice is considered outdated.

from datetime import datetime
import sys
import warnings


def read_first_non_empty_line(path: str) -> str | None:
    if path[-4:] in (".pdf", ".png", ".jpg", ".gif"):
        return None
    with open(path, encoding="utf-8") as infile:
        for line in infile:
            line = line.strip()
            if line:
                return line
    return None


curr_year = str(datetime.now().year)
for path in sys.argv[1:]:
    first_line = read_first_non_empty_line(path)
    if first_line is None:
        continue
    if "copyright" not in first_line.lower():
        warnings.warn(f"Per-file copyright notice not found for `{path}`")
        continue
    assert curr_year in first_line, (
        f"Copyright notice of file `{path}` is outdated "
        f"(year `{curr_year}` not found in the copyright notice)"
    )
