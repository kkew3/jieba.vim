# This script is meant to be run from the top-level directory of the
# repository, as 'scripts/h2h.py'. uv manages the project environment.

import argparse
import pathlib
import subprocess

import flask

import vimh2h
from h2h import get_version

root_path = pathlib.Path(vimh2h.__file__).resolve().parent


def main():
    parser = argparse.ArgumentParser(description="Convert faq markdown to HTML")
    parser.add_argument(
        "--out-dir",
        "-o",
        type=pathlib.Path,
        help="Output directory (omit for no output)",
    )
    parser.add_argument(
        "--theme",
        "-t",
        choices=("light", "dark"),
        help="Color theme for generated files (default: OS-native)",
    )
    parser.add_argument(
        "faq_file",
        type=pathlib.Path,
        help="Faq file",
    )
    args = parser.parse_args()

    app = flask.Flask(
        __name__,
        root_path=str(root_path),
    )
    app.jinja_options["trim_blocks"] = True
    app.jinja_options["lstrip_blocks"] = True
    app.jinja_env.filters["static_path"] = lambda p: p

    with app.app_context():
        run(args)


def run(args):
    content = subprocess.run(
        ["pandoc", "--from=gfm", "--to=html5", str(args.faq_file)],
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    html = flask.render_template(
        "faq.html",
        project=vimh2h.JiebaVimProject,
        version=get_version(),
        filename=args.faq_file.name,
        content=content,
        theme=args.theme,
    )
    if args.out_dir is not None:
        with (args.out_dir / f"{args.faq_file.name}.html").open(
            "w", encoding="utf-8"
        ) as f:
            f.write(html)


if __name__ == "__main__":
    main()
