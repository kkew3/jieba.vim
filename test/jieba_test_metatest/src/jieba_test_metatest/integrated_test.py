# Copyright 2026 Kaiwen Wu. All Rights Reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License. You may obtain a copy of
# the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
# WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
# License for the specific language governing permissions and limitations under
# the License.

import argparse
import contextlib
import os
import shlex
import subprocess
import sys
import uuid
from dataclasses import dataclass
from typing import Literal

from . import vimscript_transpiler as vim
from .dots_progress import DotsProgress
from .executor import pmap
from .motion_keys import WORD_MOTION_KEYS, WORD_TEXT_OBJECTS
from .parser import (
    AutocmdEventCountExpr,
    BufferExpr,
    HeadConditionalExpr,
    ParseError,
    RawBlock,
    RawDirective,
    RawTestCases,
    SourceSpan,
    StateExpr,
)


def to_tuple_opt(obj, /, len_=None) -> tuple | None:
    if obj is None:
        return None
    if len_ is not None:
        assert len(obj) == len_
    return tuple(obj)


@dataclass(unsafe_hash=True)
class IntegratedBlock:
    raw_directives: tuple[RawDirective, ...]
    # Block-level span.
    span: SourceSpan
    error_suppressed: bool

    # Head conditionals.
    hc: tuple[HeadConditionalExpr, ...]

    any_key: tuple[str, ...]

    clean_buffer_before: tuple[str, ...]
    initial_cursor: tuple[int, int, int, int, int] | None
    initial_visualmode: Literal["v", "V", "\\<C-v>"] | None
    initial_visual_begin: tuple[int, int, int, int] | None
    initial_visual_end: tuple[int, int, int, int] | None
    # States before to setup and check.
    # When generating setup code, func named "visualmode" should be ignored.
    initial_states: tuple[StateExpr, ...]

    clean_buffer_after: tuple[str, ...] | None
    result_cursor: tuple[int, int, int, int, int] | None
    result_langle: tuple[int, int, int, int] | None
    result_rangle: tuple[int, int, int, int] | None
    result_visual_begin: tuple[int, int, int, int] | None
    result_visual_end: tuple[int, int, int, int] | None
    # States to verify after the motion.
    states_to_verify: tuple[StateExpr, ...]
    # Autocmd event counts to verify after the motion.
    autocmd_event_counts_to_verify: tuple[AutocmdEventCountExpr, ...]

    @classmethod
    def from_raw_block_opt(cls, raw_block: RawBlock):
        """
        Return None if `raw_block` is not declared to export to integrated test
        block; else, construct a new integrated test block from the raw block.
        """
        if all(dr.arg != "i" for dr in raw_block.iter_directives_like("X")):
            return None

        error_suppressed = any(
            True for _ in raw_block.iter_directives_like("!")
        )

        hc = [
            HeadConditionalExpr.parse(dr.arg, dr.span)
            for dr in raw_block.iter_directives_like("?")
        ]

        any_key = [dr.arg for dr in raw_block.iter_directives_like("K")]

        initial_states = []
        initial_visualmode = None
        for dr in raw_block.iter_directives_like("S0"):
            state_expr = StateExpr.parse(dr.arg, dr.span)
            if (
                state_expr.ty == "func"
                and state_expr.name == "visualmode"
                and state_expr.value
            ):
                if state_expr.value not in {"v", "V", "\\<C-v>"}:
                    raise dr.span.to_parse_error(
                        f"invalid `S0 visualmode()` value: {state_expr.value}"
                    )
                initial_visualmode = state_expr.value
            initial_states.append(state_expr)

        buffer_before_dr = raw_block.get1("B0")
        buffer_before = BufferExpr.parse(
            buffer_before_dr.arg, buffer_before_dr.span
        )
        clean_buffer_before = buffer_before.clean_buffer
        if buffer_before.langle is not None or buffer_before.rangle is not None:
            raise buffer_before_dr.span.to_parse_error(
                "invalid position marks <, > in directive `B0`"
            )
        if initial_visualmode is not None and (
            buffer_before.visual_begin is None
            or buffer_before.visual_end is None
        ):
            raise buffer_before_dr.span.to_parse_error(
                f"missing position marks [, ] in directive `B0` "
                f"when `S0 visualmode()={initial_visualmode}`"
            )
        initial_visual_begin = buffer_before.visual_begin
        initial_visual_end = buffer_before.visual_end
        initial_cursor = buffer_before.cursor

        try:
            buffer_after_dr = raw_block.get1("B1")
        except ParseError:
            clean_buffer_after = None
            result_cursor = None
            result_langle = None
            result_rangle = None
            result_visual_begin = None
            result_visual_end = None
        else:
            buffer_after = BufferExpr.parse(
                buffer_after_dr.arg, buffer_after_dr.span
            )
            clean_buffer_after = buffer_after.clean_buffer
            result_cursor = buffer_after.cursor
            result_langle = buffer_after.langle
            result_rangle = buffer_after.rangle
            result_visual_begin = buffer_after.visual_begin
            result_visual_end = buffer_after.visual_end

        states_to_verify = []
        for dr in raw_block.iter_directives_like("S1"):
            state_expr = StateExpr.parse(dr.arg, dr.span)
            states_to_verify.append(state_expr)

        autocmd_event_counts_to_verify = [
            AutocmdEventCountExpr.parse(dr.arg, dr.span)
            for dr in raw_block.iter_directives_like("E")
        ]

        return cls(
            raw_directives=to_tuple_opt(raw_block.directives),
            span=raw_block.span,
            error_suppressed=error_suppressed,
            hc=to_tuple_opt(hc),
            any_key=to_tuple_opt(any_key),
            clean_buffer_before=to_tuple_opt(clean_buffer_before),
            initial_cursor=to_tuple_opt(initial_cursor, 5),
            initial_visualmode=initial_visualmode,
            initial_visual_begin=to_tuple_opt(initial_visual_begin, 4),
            initial_visual_end=to_tuple_opt(initial_visual_end, 4),
            initial_states=to_tuple_opt(initial_states),
            clean_buffer_after=to_tuple_opt(clean_buffer_after),
            result_cursor=to_tuple_opt(result_cursor, 5),
            result_langle=to_tuple_opt(result_langle, 4),
            result_rangle=to_tuple_opt(result_rangle, 4),
            result_visual_begin=to_tuple_opt(result_visual_begin, 4),
            result_visual_end=to_tuple_opt(result_visual_end, 4),
            states_to_verify=to_tuple_opt(states_to_verify),
            autocmd_event_counts_to_verify=to_tuple_opt(
                autocmd_event_counts_to_verify
            ),
        )

    def write_run(self, outfile):
        # Write head conditionals.
        for hc_expr in self.hc:
            if hc_expr.ty == "feature":
                outfile.write(
                    "if !has({feature})\n".format(
                        feature=vim.lit(hc_expr.value)
                    )
                )
            elif hc_expr.ty == "non_feature":
                outfile.write(
                    "if has({feature})\n".format(feature=vim.lit(hc_expr.value))
                )
            else:
                outfile.write(f"if v:version < {hc_expr.value}\n")
            _value = vim.writefile("continue", vim.sibling_file("cf"))
            outfile.write(f"{_value}\nxit\nfinish\n")
            outfile.write("endif\n")
        outfile.write("\n")

        # Define jieba mappings.
        for k in WORD_MOTION_KEYS:
            outfile.write(f"""\
nmap {k} <Plug>(Jieba_{k})
xmap {k} <Plug>(Jieba_{k})
omap {k} <Plug>(Jieba_{k})
""")
        for k in WORD_TEXT_OBJECTS:
            outfile.write(f"""\
xmap {k} <Plug>(Jieba_{k})
omap {k} <Plug>(Jieba_{k})
""")

        # Write state_before setup.
        outfile.write('" state_before setup\n')
        for state_expr in self.initial_states:
            if state_expr.ty == "mark":
                outfile.write(
                    "call setpos({key}, {value})\n".format(
                        key=vim.lit(f"'{state_expr.name}"),
                        value=vim.VimExpr.list_(state_expr.value),
                    )
                )
            elif state_expr.ty == "opt":
                outfile.write(
                    "let {lhs} = {rhs}\n".format(
                        lhs=vim.var(f"&{state_expr.name}"),
                        rhs=vim.lit(f"{state_expr.value}"),
                    )
                )
            elif state_expr.ty == "reg":
                outfile.write(
                    "call setreg({key}, {value})\n".format(
                        key=vim.lit(f"{state_expr.name}"),
                        value=vim.lit(f"{state_expr.value}"),
                    )
                )
        outfile.write("\n")

        # Write buffer_before setup.
        outfile.write('" buffer_before setup\n')
        if self.initial_visualmode is not None:
            outfile.write(
                "call setpos({key}, {value})\n".format(
                    key=vim.lit("."),
                    value=vim.VimExpr.list_(self.initial_visual_begin),
                )
            )
            outfile.write(
                "execute {cmd_str}\n".format(
                    cmd_str=vim.lit(f"normal! {self.initial_visualmode}\\<Esc>")
                )
            )
            outfile.write(
                "call setpos({key}, {value})\n".format(
                    key=vim.lit("'>"),
                    value=vim.VimExpr.list_(self.initial_visual_end),
                )
            )
        if self.initial_cursor is not None:
            outfile.write(
                "call setpos({key}, {value})\n".format(
                    key=vim.lit("."),
                    value=vim.VimExpr.list_(self.initial_cursor),
                )
            )
        outfile.write("\n")

        # Write autocmd setup.
        outfile.write('" autocmd setup\n')
        outfile.write("""\
function! IncrementAutocmdEventCount(event_name)
    let l:count = get(g:jieba_test_case_events_count, a:event_name, 0)
    let g:jieba_test_case_events_count[a:event_name] = l:count + 1
endfunction

""")
        outfile.write("augroup jieba_test_case_autocmd_events_monitoring\n")
        outfile.write("    autocmd!\n")
        for event_name in self.autocmd_event_counts_to_verify:
            outfile.write(
                "    au {event} * call IncrementAutocmdEventCount({key})\n".format(
                    event=event_name.name, key=vim.lit(event_name.name)
                )
            )
        outfile.write("augroup END\n\n")

        # Write state_before checking.
        outfile.write('" state_before checking\n')
        for state_expr in self.initial_states:
            if state_expr.ty == "func":
                outfile.write(
                    vim.not_eq_test_tofile_as_str(
                        f"unexpected state_before in function {state_expr.name}()",
                        vim.var(state_expr.name)(),
                        state_expr.value,
                        "err",
                    )
                )
            elif state_expr.ty == "mark":
                outfile.write(
                    vim.not_eq_test_tofile_as_str(
                        f"unexpected state_before in mark '{state_expr.name}",
                        vim.var("getpos")(f"'{state_expr.name}"),
                        vim.VimExpr.list_(state_expr.value),
                        "err",
                    )
                )
            elif state_expr.ty == "opt":
                outfile.write(
                    vim.not_eq_test_tofile_as_str(
                        f"unexpected state_before in option '{state_expr.name}'",
                        vim.var(f"&{state_expr.name}"),
                        state_expr.value,
                        "err",
                    )
                )
            else:  # state_expr.ty == "reg"
                outfile.write(
                    vim.not_eq_test_tofile_as_str(
                        f'unexpected state_before in register "{state_expr.name}',
                        vim.var("getreg")(state_expr.name),
                        state_expr.value,
                        "err",
                    )
                )
        outfile.write("\n")

        outfile.write("let g:jieba_test_case_events_count = {}\n")

        # Execute commands.
        outfile.write('" execute commands\n')
        for any_key_expr in self.any_key:
            any_key_expr = any_key_expr.replace("·", "\\<Space>").replace(
                "␊", "\\<CR>"
            )
            outfile.write(
                "call feedkeys({}, 't')\n".format(
                    vim.lit(f"{any_key_expr}\\<Esc>")
                )
            )

        outfile.write("function! Checks()\n")
        outfile.write(
            "let g:jieba_test_case_events_count_frozen = copy(g:jieba_test_case_events_count)\n\n"
        )

        # Autocmd event counts checking.
        outfile.write('" autocmd event counts checking\n')
        if self.autocmd_event_counts_to_verify is not None:
            for autocmd_expr in self.autocmd_event_counts_to_verify:
                outfile.write(
                    vim.not_eq_test_tofile_as_str(
                        f"unexpected autocmd_event_count for ##{autocmd_expr.name}",
                        vim.var("g:jieba_test_case_events_count_frozen")[
                            vim.lit(autocmd_expr.name)
                        ],
                        vim.int_(autocmd_expr.count),
                        "err",
                    )
                )
        outfile.write("\n")

        # State after checking.
        outfile.write('" state_after checking\n')
        for state_expr in self.states_to_verify:
            if state_expr.ty == "func":
                outfile.write(
                    vim.not_eq_test_tofile_as_str(
                        f"unexpected state_after in function {state_expr.name}()",
                        vim.var(state_expr.name)(),
                        vim.lit(state_expr.value),
                        "err",
                    )
                )
            elif state_expr.ty == "mark":
                getpos = vim.var("getpos")
                outfile.write(
                    vim.not_eq_test_tofile_as_str(
                        f"unexpected state_after in mark '{state_expr.name}",
                        getpos(f"'{state_expr.name}"),
                        vim.VimExpr.list_(state_expr.value),
                        "err",
                    )
                )
            elif state_expr.ty == "opt":
                outfile.write(
                    vim.not_eq_test_tofile_as_str(
                        f"unexpected state_after in option '{state_expr.name}'",
                        vim.var(f"&{state_expr.name}"),
                        vim.lit(state_expr.value),
                        "err",
                    )
                )
            else:  # ty == "reg":
                getreg = vim.var("getreg")
                outfile.write(
                    vim.not_eq_test_tofile_as_str(
                        f'unexpected state_after in register "{state_expr.name}',
                        getreg(vim.lit(state_expr.name)),
                        vim.lit(state_expr.value),
                        "err",
                    )
                )
        outfile.write("\n")

        # Buffer after checking.
        outfile.write('" buffer_after checking\n')
        getcurpos = vim.var("getcurpos")
        getpos = vim.var("getpos")
        if self.result_cursor is not None:
            outfile.write(
                vim.not_eq_test_tofile_as_str(
                    "unexpected cursor position in buffer_after",
                    getcurpos(),
                    vim.VimExpr.list_(self.result_cursor),
                    "err",
                )
            )
        if self.result_langle is not None:
            outfile.write(
                vim.not_eq_test_tofile_as_str(
                    "unexpected '< position in buffer_after",
                    getpos(vim.lit("'<")),
                    vim.VimExpr.list_(self.result_langle),
                    "err",
                )
            )
        if self.result_rangle is not None:
            outfile.write(
                vim.not_eq_test_tofile_as_str(
                    "unexpected '> position in buffer_after",
                    getpos(vim.lit("'>")),
                    vim.VimExpr.list_(self.result_rangle),
                    "err",
                )
            )
        if (
            self.result_visual_begin is not None
            or self.result_visual_end is not None
        ):
            outfile.write("normal! gvomaomb\n")
            if self.result_visual_begin is not None:
                outfile.write(
                    vim.not_eq_test_tofile_as_str(
                        "unexpected visual_begin position in buffer_after",
                        getpos(vim.lit("'a")),
                        vim.VimExpr.list_(self.result_visual_begin),
                        "err",
                    )
                )
            if self.result_visual_end is not None:
                outfile.write(
                    vim.not_eq_test_tofile_as_str(
                        "unexpected visual_end position in buffer_after",
                        getpos(vim.lit("'b")),
                        vim.VimExpr.list_(self.result_visual_end),
                        "err",
                    )
                )
        outfile.write("\n")
        # FIXME Dummy line that adds one more Ex loop in Vim. Without this the
        #       test vimscript will return 1 silently after passing all checks.
        #       Perhaps there's a cleverer way to solve this problem.
        outfile.write('call feedkeys(":\\<C-u>\\<CR>", "n")\n')

        outfile.write("endfunction\n")

        # Run checks.
        outfile.write('call feedkeys(":\\<C-u>call Checks()\\<CR>", "nt")\n')
        # FIXME Dummy line that adds one more Ex loop in Vim. Without this the
        #       test vimscript will return 1 silently after passing all checks.
        #       Perhaps there's a cleverer way to solve this problem.
        outfile.write('call feedkeys(":\\<C-u>\\<CR>", "n")\n')

        # Exit.
        outfile.write('call feedkeys(":\\<C-u>silent xit\\<CR>", "nt")\n')

    def run_test(
        self,
        vimrc: str | None,
        work_dir: str,
        vim_bin: str | None,
        vim_type: Literal["vim", "nvim"],
    ) -> 'None | Literal["continue", "dry_run"] | IntegratedTestFailure':
        if (
            any((dr.ty, dr.value) == ("non_feature", "nvim") for dr in self.hc)
            and vim_type == "nvim"
        ) or (
            any((dr.ty, dr.value) == ("feature", "nvim") for dr in self.hc)
            and vim_type == "vim"
        ):
            # Shortcut path for mismatched runtime.
            return "continue"

        os.mkdir(work_dir)  # may raise FileExistsError, which is intentional

        buffer_file = os.path.join(work_dir, "buffer")
        with open(buffer_file, "w", encoding="utf-8") as outfile:
            for line in self.clean_buffer_before:
                outfile.write(f"{line}\n")
        run_file = os.path.join(work_dir, "run.vim")
        with open(run_file, "w", encoding="utf-8") as outfile:
            self.write_run(outfile)

        # Run test.
        cmd = [vim_bin or vim_type]  # If vim_bin is None, will use vim_type.
        if vim_type == "vim":
            cmd.append("--not-a-term")
        if vimrc is not None:
            cmd.extend(["-u", vimrc])
        cmd.extend(["-S", run_file])
        cmd.append(buffer_file)

        if vim_bin is None:
            # Dry-run path.
            cmd = [shlex.quote(x) for x in cmd]
            print(">", *cmd)
            return "dry_run"

        proc = subprocess.run(
            cmd,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            timeout=5,
        )
        if proc.returncode != 0:
            try:
                err_file = os.path.join(work_dir, "err")
                with open(err_file, encoding="utf-8") as infile:
                    stderr = infile.read()
            except FileNotFoundError:
                stderr = ""
            return IntegratedTestFailure(
                self.span, stderr, self.error_suppressed
            )

        try:
            cf_file = os.path.join(work_dir, "cf")
            with open(cf_file, encoding="utf-8") as infile:
                if infile.read().strip() == "continue":
                    return "continue"
        except FileNotFoundError:
            pass

        if self.clean_buffer_after is not None:
            with open(buffer_file, encoding="utf-8") as infile:
                actual_buffer_after = [line.rstrip("\n") for line in infile]
            if actual_buffer_after != list(self.clean_buffer_after):
                pretty_expected = BufferExpr.pprint_clean_buffer(
                    self.clean_buffer_after
                )
                pretty_actual = BufferExpr.pprint_clean_buffer(
                    actual_buffer_after
                )
                return IntegratedTestFailure(
                    self.span,
                    (
                        f"expected buffer_after:\n\n{pretty_expected}\n"
                        f"actual buffer_after:\n\n{pretty_actual}"
                    ),
                    self.error_suppressed,
                )

        # Test passed.
        return None


@dataclass
class IntegratedTestFailure:
    block_span: SourceSpan
    message: str
    error_suppressed: bool

    def __str__(self):
        return f"i case failed: {self.block_span} -->\n{self.message}"


def make_parser():
    parser = argparse.ArgumentParser(description="Integrated test cli.")
    parser.add_argument(
        "--rc",
        dest="vimrc",
        help=(
            "The vimrc path for basic integrated verification. "
            "If using vim instance from docker container where "
            "vimrc has been baked in, this option may not be necessary"
        ),
    )
    parser.add_argument(
        "-v",
        dest="vim_bin",
        help=(
            "The full path or PATH-searchable name of vim/nvim binary. "
            "Leave this unspecified to enable dry-run mode, in which "
            "the run script etc. can be inspected."
        ),
    )
    parser.add_argument(
        "--neovim",
        action="store_true",
        dest="is_neovim",
        help="Specify this if `-v` points to neovim.",
    )
    parser.add_argument(
        "-d",
        dest="work_dir",
        help=(
            "The working directory under which to run unit test verifications."
        ),
    )
    parser.add_argument(
        "-j",
        dest="n_jobs",
        type=int,
        default=0,
        help=(
            "Number of jobs to run (>=1). Default to 0. "
            "Pass 0 to run sequentially."
        ),
    )
    parser.add_argument(
        "-f", dest="err_file", help="Tee failure report to this file."
    )
    parser.add_argument(
        "-w",
        dest="warn_file",
        help="Tee failure report for suppressed errors to this file.",
    )
    parser.add_argument(
        "test_case_file", nargs="*", help="The *.jieba_test_case files."
    )
    return parser


def main():
    args = make_parser().parse_args()
    os.makedirs(args.work_dir, exist_ok=True)
    vim_type = "nvim" if args.is_neovim else "vim"
    visited_case = set()
    suppressed_errors = []
    if args.err_file is None:
        err_fileobj = contextlib.nullcontext()
    else:
        err_fileobj = open(args.err_file, "w", encoding="utf-8")

    # Write a .gitignore under args.work_dir to ensure it won't be indexed by
    # any git client (e.g. VSCode, Fork), as there will could be a HUGE number
    # of folders created under it and those client will definitely get stuck.
    ignore_file = os.path.join(args.work_dir, ".gitignore")
    with open(ignore_file, "w", encoding="utf-8") as outfile:
        outfile.write("/*\n")

    def setup_fn(_c: IntegratedBlock):
        if _c in visited_case:
            print(
                f"W: dup detected: ignored test case {_c.span}",
                file=sys.stderr,
            )
            return False
        visited_case.add(_c)
        case_id = uuid.uuid4().hex
        return (case_id,)

    def runner(_c: IntegratedBlock, case_id, vimrc, vim_bin, vim_type):
        case_work_dir = os.path.join(args.work_dir, case_id)
        return _c.run_test(vimrc, case_work_dir, vim_bin, vim_type)

    for path in args.test_case_file:
        raw_cases = RawTestCases()
        try:
            raw_cases.extend_from_file(path)
        except (FileNotFoundError, OSError):
            print(f"io warning: file unreadable: {path}", file=sys.stderr)
            continue
        print(f"I: {path}: found {len(raw_cases)} raw test cases")
        i_blocks = list(
            filter(None, map(IntegratedBlock.from_raw_block_opt, raw_cases))
        )
        print(f"I: {path}: found {len(i_blocks)} i blocks")
        if args.vim_bin is None:
            print("I: dry-run mode")

        with DotsProgress() as progress:
            for _, fut in pmap(
                setup_fn,
                runner,
                i_blocks,
                args.n_jobs,
                vimrc=args.vimrc,
                vim_bin=args.vim_bin,
                vim_type=vim_type,
            ):
                excp = fut.exception()
                if excp is not None:
                    print(f"E: {excp}", file=sys.stderr)
                    sys.exit(127)
                res = fut.result()
                if args.vim_bin is None:
                    assert res == "dry_run"
                    progress.step()
                    continue
                if isinstance(res, IntegratedTestFailure):
                    if res.error_suppressed:
                        suppressed_errors.append(res)
                        progress.step(err=True)
                        continue
                    print(f"F: {res}", file=sys.stderr)
                    if args.err_file is not None:
                        with open(
                            args.err_file, "a", encoding="utf-8"
                        ) as err_fileobj:
                            err_fileobj.write(f"{res}\n")
                    sys.exit(1)
                progress.step()

    if args.warn_file is None:
        warn_fileobj = contextlib.nullcontext()
    else:
        warn_fileobj = open(args.warn_file, "a", encoding="utf-8")
    if suppressed_errors:
        print("Suppressed failures:")
        with warn_fileobj:
            for res in suppressed_errors:
                print("---")
                print(f"F: {res}")
                if args.warn_file is not None:
                    warn_fileobj.write(f"{res}\n\n")
        sys.exit(125)
