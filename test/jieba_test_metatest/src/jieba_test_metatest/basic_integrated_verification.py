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
import concurrent.futures
import json
import os
import shlex
import string
import subprocess
import sys
import uuid
from dataclasses import dataclass
from typing import Literal

from . import vimscript_transpiler as vim
from .dots_progress import DotsProgress
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


def get1(raw_block: RawBlock, dr_type: str) -> RawDirective:
    """Get the first one from `directives` and raise error if there're more."""
    first = None
    for dr in raw_block.iter_directives_like(dr_type):
        if first is None:
            first = dr
            continue
        raise dr.span.to_parse_error(
            f"expecting exactly one arg for directive `{first.ty}` "
            f"but found more"
        )
    if first is None:
        raise raw_block.span.to_parse_error(
            f"expecting exactly one arg for directive `{dr_type}` "
            f"but found none"
        )
    return first


def to_tuple_opt(obj, /, len_=None) -> tuple | None:
    if obj is None:
        return None
    if len_ is not None:
        assert len(obj) == len_
    return tuple(obj)


def is_valid_motion_key(motion_key_value: str) -> bool:
    return motion_key_value in {
        "w",
        "W",
        "e",
        "E",
        "b",
        "B",
        "ge",
        "gE",
        "iw",
        "iW",
        "aw",
        "aW",
    }


@dataclass(unsafe_hash=True)
class BasicIntegratedBlock:
    raw_directives: tuple[RawDirective, ...]
    # Block-level span.
    span: SourceSpan

    # Head conditionals.
    hc: tuple[HeadConditionalExpr, ...]

    mode: Literal["n", "x", "o"]
    motion_key: str
    # Either a positive integer as string or empty.
    count: str
    # If mode is not "o", this will be None.
    operator: str | None
    # If mode is not "o", this will be None. When this is not None, an empty
    # string value denotes the default implicit register.
    register: str | None

    clean_buffer_before: tuple[str, ...]

    initial_visualmode: Literal["v", "V", "\\<C-v>"] | None
    initial_visual_begin: tuple[int, int, int, int] | None
    initial_visual_end: tuple[int, int, int, int] | None

    # If mode is "x", this may be None.
    initial_cursor: tuple[int, int, int, int, int] | None

    # States before to setup and check.
    # When generating setup code, func named "visualmode" should be ignored.
    initial_states: tuple[StateExpr, ...]
    # States to verify after the motion.
    states_to_verify: tuple[StateExpr, ...]
    # Autocmd event counts to verify after the motion.
    autocmd_events_to_verify: tuple[str, ...]

    @classmethod
    def from_raw_block_opt(cls, raw_block: RawBlock):
        """
        Return None if `raw_block` is not declared to export to basic
        integrated verification block; else, construct a new basic integrated
        verification block from the raw block.
        """
        if all(dr.arg != "bi" for dr in raw_block.iter_directives_like("X")):
            return None

        hc = [
            HeadConditionalExpr.parse(dr.arg, dr.span)
            for dr in raw_block.iter_directives_like("?")
        ]

        mode_dr = get1(raw_block, "M")
        if mode_dr.arg not in {"n", "o", "v", "V", "\\<C-v>"}:
            raise mode_dr.span.to_parse_error(
                f"invalid directive `M` value: {mode_dr.arg}"
            )
        tr = {"n": "n", "o": "o", "v": "x", "V": "x", "\\<C-v>": "x"}
        mode = tr[mode_dr.arg]

        motion_key_dr = get1(raw_block, "K")
        if not is_valid_motion_key(motion_key_dr.arg):
            raise motion_key_dr.span.to_parse_error(
                f"invalid directive `K` value: {motion_key_dr.arg}"
            )
        if mode == "n" and motion_key_dr.arg in {"iw", "iW", "aw", "aW"}:
            raise motion_key_dr.span.to_parse_error(
                f"invalid directive `K` value when `M n`: {motion_key_dr.arg}"
            )
        motion_key = motion_key_dr.arg

        try:
            count_dr = get1(raw_block, "C")
            count = str(int(count_dr.arg))
            if count == "0":
                count = ""
        except (ParseError, ValueError):
            count = ""

        if mode == "o":
            operator = get1(raw_block, "O").arg
        else:
            operator = None

        if mode == "o":
            try:
                register_dr = get1(raw_block, "R")
            except ParseError:
                register = ""
            else:
                if len(register_dr.arg) != 1:
                    raise register_dr.span.to_parse_error(
                        f"invalid directive `R` value: {register_dr.arg}"
                    )
                register = register_dr.arg
        else:
            register = None

        initial_states = []
        initial_visualmode = None
        for dr in raw_block.iter_directives_like("S0"):
            state_expr = StateExpr.parse(dr.arg, dr.span)
            if (
                state_expr.ty == "func"
                and state_expr.name == "visualmode"
                and state_expr.value
            ):
                if mode == "x" and state_expr.value != mode_dr.arg:
                    raise dr.span.to_parse_error(
                        f"`S0 visualmode()={state_expr.value}` "
                        f"inconsistent with `M {mode_dr.arg}`"
                    )
                if state_expr.value not in {"v", "V", "\\<C-v>"}:
                    raise dr.span.to_parse_error(
                        f"invalid `S0 visualmode()` value: {state_expr.value}"
                    )
                initial_visualmode = state_expr.value
            initial_states.append(state_expr)
        if mode == "x" and initial_visualmode is None:
            initial_visualmode = mode_dr.arg

        buffer_before_dr = get1(raw_block, "B0")
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

        if mode in ("n", "o") and buffer_before.cursor is None:
            raise buffer_before_dr.span.to_parse_error(
                "missing position mark | in directive `B0` "
                "when mode is 'n' or 'o'"
            )
        initial_cursor = buffer_before.cursor

        states_to_verify = []
        for dr in raw_block.iter_directives_like("S1"):
            state_expr = StateExpr.parse(
                dr.arg, dr.span, parse_as_incomplete=True
            )
            if (
                state_expr.ty == "mark"
                and state_expr.name not in string.ascii_lowercase
                and state_expr.name not in {"<", ">", "[", "]"}
            ):
                raise dr.span.to_parse_error(
                    f"unsupported mark `{state_expr.name}`"
                )
            elif (
                state_expr.ty == "reg"
                and state_expr.name not in string.ascii_lowercase
                and state_expr.name != '"'
            ):
                raise dr.span.to_parse_error(
                    f"unsupported register `{state_expr.name}`"
                )
            states_to_verify.append(state_expr)

        autocmd_events_to_verify = [
            AutocmdEventCountExpr.parse(
                dr.arg, dr.span, parse_as_incomplete=True
            ).name
            for dr in raw_block.iter_directives_like("E")
        ]

        return cls(
            raw_directives=to_tuple_opt(raw_block.directives),
            span=raw_block.span,
            hc=to_tuple_opt(hc),
            mode=mode,
            motion_key=motion_key,
            count=count,
            operator=operator,
            register=register,
            clean_buffer_before=to_tuple_opt(clean_buffer_before),
            initial_visualmode=initial_visualmode,
            initial_visual_begin=to_tuple_opt(initial_visual_begin, 4),
            initial_visual_end=to_tuple_opt(initial_visual_end, 4),
            initial_cursor=to_tuple_opt(initial_cursor, 5),
            initial_states=to_tuple_opt(initial_states),
            states_to_verify=to_tuple_opt(states_to_verify),
            autocmd_events_to_verify=to_tuple_opt(autocmd_events_to_verify),
        )

    def write_head_conditionals(self, outfile):
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
            cf_dict = vim.VimExpr.dict_({"cf": "continue"})
            _value_lua = vim.echo(
                False, vim.json_encoded(vim.LuaExpr.wrap_vim(cf_dict))
            )
            _value_vim = vim.echo(False, vim.json_encoded(cf_dict))
            outfile.write(f"""\
    if has("nvim")
        lua <<EOF
{_value_lua}
EOF
    else
        {_value_vim}
    endif
""")
            outfile.write("endif\n")
        outfile.write("\n")

    def write_vimscript_setup(self, outfile):
        # Define oracle model.
        func = {
            "n": "JiebaModelNmap",
            "x": "JiebaModelXmap",
            "o": "JiebaModelOmap",
        }[self.mode]
        outfile.write(f"""\
" define oracle model
function! JiebaOracleModel(...)
    let g:model_input = a:000
    let g:model_output = call(function("{func}"), a:000)
    return g:model_output
endfunction

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
        for event_name in self.autocmd_events_to_verify:
            outfile.write(
                "    au {event} * call IncrementAutocmdEventCount({key})\n".format(
                    event=event_name, key=vim.lit(event_name)
                )
            )
        outfile.write("augroup END\n\n")

        # Write state_before checking.
        outfile.write('" state_before checking\n')
        for state_expr in self.initial_states:
            if state_expr.ty == "func":
                outfile.write(
                    vim.not_eq_test_as_str(
                        f"unexpected state_before in function {state_expr.name}()",
                        vim.var(state_expr.name)(),
                        state_expr.value,
                    )
                )
            elif state_expr.ty == "mark":
                outfile.write(
                    vim.not_eq_test_as_str(
                        f"unexpected state_before in mark '{state_expr.name}",
                        vim.var("getpos")(f"'{state_expr.name}"),
                        vim.VimExpr.list_(state_expr.value),
                    )
                )
            elif state_expr.ty == "opt":
                outfile.write(
                    vim.not_eq_test_as_str(
                        f"unexpected state_before in option '{state_expr.name}'",
                        vim.var(f"&{state_expr.name}"),
                        state_expr.value,
                    )
                )
            else:  # state_expr.ty == "reg"
                outfile.write(
                    vim.not_eq_test_as_str(
                        f'unexpected state_before in register "{state_expr.name}',
                        vim.var("getreg")(state_expr.name),
                        state_expr.value,
                    )
                )

    def write_std_run(self, outfile):
        self.write_head_conditionals(outfile)

        # Setup.
        self.write_vimscript_setup(outfile)
        outfile.write("\n\n")

        outfile.write("let g:jieba_test_case_events_count = {}\n")

        # Cursor movement.
        outfile.write('" cursor movement\n')
        if self.mode == "n":
            outfile.write(f"normal! {self.count}{self.motion_key}\n")
        elif self.mode == "x":
            outfile.write(f"normal! gv{self.count}{self.motion_key}\n")
        else:
            reg = f'"{self.register}' if self.register else ""
            outfile.write(
                f"normal! {reg}{self.operator}{self.count}{self.motion_key}\n"
            )
        outfile.write('execute "normal! \\<Esc>"\n\n')
        outfile.write(
            "let s:jieba_test_case_events_count_frozen = copy(g:jieba_test_case_events_count)\n\n"
        )

        # Autocmd event counts querying.
        outfile.write("""\
" autocmd event counts querying
let g:JiebaTestGroundtruthAutocmdEventsCount = json_encode(s:jieba_test_case_events_count_frozen)

""")

        # State after querying.
        outfile.write('" state_after querying\n')
        for state_expr in self.states_to_verify:
            if state_expr.ty == "func":
                outfile.write(
                    f"let g:JiebaTestGroundtruthFunc_{state_expr.name} = {state_expr.name}()\n"
                )
            elif state_expr.ty == "mark":
                if state_expr.name == "<":
                    _v = "JiebaTestGroundtruthMark_langle"
                elif state_expr.name == ">":
                    _v = "JiebaTestGroundtruthMark_rangle"
                elif state_expr.name == "[":
                    _v = "JiebaTestGroundtruthMark_lsquare"
                elif state_expr.name == "]":
                    _v = "JiebaTestGroundtruthMark_rsquare"
                else:
                    _v = f"JiebaTestGroundtruthMark_{state_expr.name}"
                outfile.write(
                    f'let g:{_v} = json_encode(getpos("\'{state_expr.name}"))\n'
                )
            elif state_expr.ty == "opt":
                outfile.write(
                    f"let g:JiebaTestGroundtruthOption_{state_expr.name} = &{state_expr.name}\n"
                )
            else:  # ty == "reg"
                if state_expr.name == '"':
                    _v = "JiebaTestGroundtruthReg_default"
                else:
                    _v = f"JiebaTestGroundtruthReg_{state_expr.name}"
                outfile.write(f"let g:{_v} = getreg('{state_expr.name}')\n")
        outfile.write("\n")

        # Buffer after querying and echoing.
        outfile.write('" buffer_after querying\n')
        getcurpos = vim.var("getcurpos")
        json_encode = vim.var("json_encode")
        outfile.write(
            f"let g:JiebaTestGroundtruthCursor = {json_encode(getcurpos())}\n"
        )
        if self.mode == "x":
            outfile.write("""\
normal! gvomaomb
let g:JiebaTestGroundtruthVisualBegin = json_encode(getpos("'a"))
let g:JiebaTestGroundtruthVisualEnd = json_encode(getpos("'b"))
""")
        outfile.write("\n")

        # Make session and exit.
        outfile.write("""\
execute "mksession! " . expand("%:p:h") . "/Session.vim"
silent xit
""")

    def write_custom_run(self, outfile):
        self.write_head_conditionals(outfile)

        # Load session.
        outfile.write("""\
silent execute "source " . expand("%:p:h") . "/Session.vim"

""")

        # Setup.
        self.write_vimscript_setup(outfile)
        outfile.write("\n\n")

        outfile.write("let g:jieba_test_case_events_count = {}\n")

        # Cursor movement.
        outfile.write('" cursor movement\n')
        if self.mode == "n":
            outfile.write(
                'call JiebaNmap({motion_key}, {count}, "JiebaOracleModel")\n'.format(
                    motion_key=vim.lit(self.motion_key), count=self.count or "0"
                )
            )
        elif self.mode == "x":
            outfile.write(
                'call JiebaXmap({motion_key}, {count}, "JiebaOracleModel")\n'.format(
                    motion_key=vim.lit(self.motion_key), count=self.count or "0"
                )
            )
        else:
            outfile.write(
                'call JiebaOmap({motion_key}, 0, {count}, {operator}, {register}, "JiebaOracleModel")\n'.format(
                    motion_key=vim.lit(self.motion_key),
                    count=self.count or "0",
                    operator=vim.lit(self.operator),
                    register=(
                        vim.lit(self.register)
                        if self.register
                        else vim.lit('"')
                    ),
                )
            )
        outfile.write('execute "normal! \\<Esc>"\n\n')
        outfile.write(
            "let g:jieba_test_case_events_count_frozen = copy(g:jieba_test_case_events_count)\n\n"
        )

        # Autocmd event counts checking.
        outfile.write('" autocmd event counts checking\n')
        json_decode = vim.var("json_decode")
        outfile.write(
            vim.not_eq_test_as_str(
                "unexpected autocmd events count",
                vim.var("g:jieba_test_case_events_count_frozen"),
                json_decode(
                    vim.var("g:JiebaTestGroundtruthAutocmdEventsCount")
                ),
            )
        )
        outfile.write("\n")

        # State after checking.
        outfile.write('" state_after checking\n')
        for state_expr in self.states_to_verify:
            if state_expr.ty == "func":
                outfile.write(
                    vim.not_eq_test_as_str(
                        f"unexpected state_after in function {state_expr.name}()",
                        vim.var(state_expr.name)(),
                        vim.var(
                            f"g:JiebaTestGroundtruthFunc_{state_expr.name}"
                        ),
                    )
                )
            elif state_expr.ty == "mark":
                if state_expr.name in string.ascii_lowercase:
                    _v = f"JiebaTestGroundtruthMark_{state_expr.name}"
                else:
                    _v = {
                        "<": "JiebaTestGroundtruthMark_langle",
                        ">": "JiebaTestGroundtruthMark_rangle",
                        "[": "JiebaTestGroundtruthMark_lsquare",
                        "]": "JiebaTestGroundtruthMark_rsquare",
                    }[state_expr.name]
                outfile.write(
                    vim.not_eq_test_as_str(
                        f"unexpected state_after in mark '{state_expr.name}",
                        vim.var("getpos")(f"'{state_expr.name}"),
                        vim.var("json_decode")(vim.var(f"g:{_v}")),
                    )
                )
            elif state_expr.ty == "opt":
                outfile.write(
                    vim.not_eq_test_as_str(
                        f"unexpected state_after in option '{state_expr.name}'",
                        vim.var(f"&{state_expr.name}"),
                        vim.var(
                            f"g:JiebaTestGroundtruthOption_{state_expr.name}"
                        ),
                    )
                )
            else:  # ty == "reg"
                if state_expr.name in string.ascii_lowercase:
                    _v = f"JiebaTestGroundtruthReg_{state_expr.name}"
                else:
                    _v = {'"': "JiebaTestGroundtruthReg_default"}[
                        state_expr.name
                    ]
                outfile.write(
                    vim.not_eq_test_as_str(
                        f'unexpected state_after in register "{state_expr.name}',
                        vim.var("getreg")(state_expr.name),
                        vim.var(f"g:{_v}"),
                    )
                )
        outfile.write("\n")

        # Buffer after checking.
        outfile.write('" buffer_after checking\n')
        getcurpos = vim.var("getcurpos")
        json_decode = vim.var("json_decode")
        getpos = vim.var("getpos")
        outfile.write(
            vim.not_eq_test_as_str(
                "unexpected cursor position in buffer_after",
                getcurpos(),
                json_decode(vim.var("g:JiebaTestGroundtruthCursor")),
            )
        )
        if self.mode == "x":
            outfile.write("normal! gvomaomb\n")
            outfile.write(
                vim.not_eq_test_as_str(
                    "unexpected visual_begin position in buffer_after",
                    getpos(vim.lit("'a")),
                    json_decode(vim.var("g:JiebaTestGroundtruthVisualBegin")),
                )
            )
            outfile.write(
                vim.not_eq_test_as_str(
                    "unexpected visual_end position in buffer_after",
                    getpos(vim.lit("'b")),
                    json_decode(vim.var("g:JiebaTestGroundtruthVisualEnd")),
                )
            )
        outfile.write("\n")

        # Model output echoing.
        outfile.write('" model_output echoing\n')
        io_dict = vim.VimExpr.dict_(
            {"i": vim.var("g:model_input"), "o": vim.var("g:model_output")}
        )
        _value_lua = vim.echo(
            False,
            vim.json_encoded(vim.LuaExpr.wrap_vim(io_dict)),
        )
        _value_vim = vim.echo(False, vim.json_encoded(io_dict))
        outfile.write(f"""\
if has("nvim")
    lua <<EOF
{_value_lua}
EOF
else
    {_value_vim}
endif

""")

        # Exit.
        outfile.write("silent xit\n")

    def run_verification(
        self,
        vimrc: str | None,
        work_dir: str,
        vim_bin: str | None,
        vim_type: Literal["vim", "nvim"],
    ) -> 'BasicIntegratedVerificationFailure | VerificationOutput | Literal["continue", "dry_run"]':
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

        # Std-run.
        buffer_file = os.path.join(work_dir, "buffer")
        with open(buffer_file, "w", encoding="utf-8") as outfile:
            for line in self.clean_buffer_before:
                outfile.write(f"{line}\n")
        std_run_file = os.path.join(work_dir, "std_run.vim")
        with open(std_run_file, "w", encoding="utf-8") as outfile:
            self.write_std_run(outfile)
        resp = verify_in_vim(
            vim_bin,
            vimrc,
            std_run_file,
            buffer_file,
            expected_buffer_after=None,
            run_type="std-run",
            vim_type=vim_type,
            block_span=self.span,
        )
        if resp == "continue":
            return "continue"
        if isinstance(resp, BasicIntegratedVerificationFailure):
            return resp
        with open(buffer_file, encoding="utf-8") as infile:
            expected_buffer_after = [line.rstrip("\n") for line in infile]

        # Custom-run.
        with open(buffer_file, "w", encoding="utf-8") as outfile:
            for line in self.clean_buffer_before:
                outfile.write(f"{line}\n")
        custom_run_file = os.path.join(work_dir, "custom_run.vim")
        with open(custom_run_file, "w", encoding="utf-8") as outfile:
            self.write_custom_run(outfile)
        resp = verify_in_vim(
            vim_bin,
            vimrc,
            custom_run_file,
            buffer_file,
            expected_buffer_after,
            run_type="custom-run",
            vim_type=vim_type,
            block_span=self.span,
        )
        assert resp is not None, "unreachable"
        if resp == "continue":
            return "continue"
        if resp == "dry_run":
            return "dry_run"
        if isinstance(resp, BasicIntegratedVerificationFailure):
            return resp

        # Collect into outputs.
        return VerificationOutput(
            fun_name=f"{self.mode}map",
            buffer=list(self.clean_buffer_before),
            model_input=resp.input,
            model_output=resp.output,
            span=f"{self.span}",
        )


@dataclass
class VimRunResponse:
    # Alias: "i".
    input: list
    # Alias: "o".
    output: dict


def pretty_print_clean_buffer(clean_buffer: list[str]) -> str:
    if clean_buffer:
        return "".join(
            line.replace(" ", "·").replace("\t", "┤") + "␊\n"
            for line in clean_buffer
        )
    return "␀\n"


@dataclass
class BasicIntegratedVerificationFailure:
    run_type: Literal["std-run", "custom-run"]
    block_span: SourceSpan
    message: str

    def __str__(self):
        return (
            f"bi case failed ({self.run_type}): {self.block_span} -->\n"
            f"{self.message}"
        )


def verify_in_vim(
    vim_bin: str | None,
    vimrc: str | None,
    run_file: str,
    buffer_file: str,
    expected_buffer_after: list[str] | None,
    run_type: Literal["std-run", "custom-run"],
    vim_type: Literal["vim", "nvim"],
    block_span: SourceSpan,
) -> (
    VimRunResponse
    | BasicIntegratedVerificationFailure
    | Literal["continue", "dry_run"]
    | None
):
    """
    If `vim_bin` is None, will run in dry-run mode and return
    "dry_run"; else, if head conditionals failed (`control_flow` ==
    "continue"), return "continue"; else, if verification failed, return
    BasicIntegratedVerificationFailure; else, if `run_type` equals "std-run",
    will also return None; else, return VimRunResponse.

    If `vimrc` is not None, will run with that vimrc.
    """
    cmd = [vim_bin or vim_type]  # If vim_bin is None, will use vim_type.
    if vim_type == "vim":
        cmd.append("-es")
    else:
        cmd.append("--headless")
    if vimrc is not None:
        cmd.extend(["-u", vimrc])
    cmd.extend(["-S", run_file])
    cmd.append(buffer_file)

    if vim_bin is None:
        # Dry-run path.
        cmd = [shlex.quote(x) for x in cmd]
        print(">", *cmd)
        return "dry_run"

    env = os.environ.copy()
    env["JIEBA_TEST_CASE"] = "1"
    proc = subprocess.run(
        cmd,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=env,
        timeout=5,
    )
    if proc.returncode != 0:
        return BasicIntegratedVerificationFailure(
            run_type, block_span, proc.stderr
        )

    if run_type == "std-run" and not proc.stdout:
        # In std-run and there is no head conditionals, the stdout will be
        # empty. In this case it's safe to skip json decoding of the stdout.
        msg = {}
    else:
        msg = json.loads(proc.stdout)  # `msg` should be a dict
    if msg.get("cf", None) == "continue":
        return "continue"

    if expected_buffer_after is not None:
        with open(buffer_file, encoding="utf-8") as infile:
            actual_buffer_after = [line.rstrip("\n") for line in infile]
        if actual_buffer_after != expected_buffer_after:
            pretty_expected = pretty_print_clean_buffer(expected_buffer_after)
            pretty_actual = pretty_print_clean_buffer(actual_buffer_after)
            return BasicIntegratedVerificationFailure(
                run_type,
                block_span,
                (
                    f"expected buffer_after:\n\n{pretty_expected}\n"
                    f"actual buffer_after:\n\n{pretty_actual}"
                ),
            )

    if run_type == "std-run":
        return None

    if "i" not in msg or "o" not in msg:
        raise ValueError(f"model i/o is None: {block_span}")

    return VimRunResponse(input=msg["i"], output=msg["o"])


@dataclass
class VerificationOutput:
    # Alias: "f".
    fun_name: Literal["nmap", "xmap", "omap"]
    # Alias: "b". Clean buffer_before.
    buffer: list[str]
    # Alias: "i". Model inputs.
    model_input: list
    # Alias: "o". Model outputs.
    model_output: dict
    # Block span as str.
    span: str


def make_parser():
    parser = argparse.ArgumentParser(
        description="Basic integrated verification cli."
    )
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
        "-n",
        dest="vim_dist_name",
        help=(
            "The vim/nvim distribution name. Default to the last component "
            "of `-v`, or 'vim' if `-v` is not provided. The caller needs to "
            "ensure that the name contains only characters that are safe to "
            "be used in a file base name."
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
        default=1,
        help=(
            "Number of jobs to run (>=1). Default to 1. "
            "Pass 0 to run sequentially."
        ),
    )
    parser.add_argument(
        "test_case_file", nargs="*", help="The *.jieba_test_case files."
    )
    return parser


class FutureWrapper:
    def __init__(self, res, excp):
        self.res = res
        self.excp = excp

    def exception(self):
        return self.excp

    def result(self):
        return self.res


def pmap(setup_fn, runner, data, n_jobs, **kwargs):
    """
    `setup_fn`, if not None, should be a callable that takes each item in
    data as argument and returns either False if the item should be skipped
    from enqueueing, or a tuple to be bound with the result of the item. Then,
    `runner` will be called like `runner(item, *setup_data, **kwargs)`.
    """
    assert n_jobs >= 0
    if n_jobs == 0:
        for item in data:
            if setup_fn is not None:
                setup_data = setup_fn(item)
                if not setup_data:
                    continue
            else:
                setup_data = ()
            try:
                res = runner(item, *setup_data, **kwargs)
                yield (setup_data, FutureWrapper(res, excp=None))
            except Exception as excp:
                yield (setup_data, FutureWrapper(res=None, excp=excp))
    else:
        executor = concurrent.futures.ThreadPoolExecutor(n_jobs)
        try:
            fs = {}
            for item in data:
                if setup_fn is not None:
                    setup_data = setup_fn(item)
                    if not setup_data:
                        continue
                else:
                    setup_data = ()
                _fut = executor.submit(runner, item, *setup_data, **kwargs)
                fs[_fut] = setup_data
            for fut in concurrent.futures.as_completed(fs):
                yield (fs[fut], fut)
        finally:
            executor.shutdown(cancel_futures=True)  # requires python>=3.9


def main():
    args = make_parser().parse_args()
    os.makedirs(args.work_dir, exist_ok=True)
    vim_dist_name = args.vim_dist_name or os.path.basename(
        args.vim_bin or "vim"
    )
    vim_type = "nvim" if args.is_neovim else "vim"
    unit_info_file = os.path.join(args.work_dir, f"unit-{vim_dist_name}.jsonl")
    written_to_unit_info = False
    visited_case = set()

    def setup_fn(_c: BasicIntegratedBlock):
        if _c in visited_case:
            print(
                f"W: dup detected: ignored test case {_c.span}",
                file=sys.stderr,
            )
            return False
        visited_case.add(_c)
        case_id = uuid.uuid4().hex
        return (case_id,)

    def runner(_c: BasicIntegratedBlock, case_id, vimrc, vim_bin, vim_type):
        case_work_dir = os.path.join(args.work_dir, case_id)
        return _c.run_verification(vimrc, case_work_dir, vim_bin, vim_type)

    with open(unit_info_file, "w", encoding="utf-8") as outfile:
        for path in args.test_case_file:
            raw_cases = RawTestCases()
            raw_cases.extend_from_file(path)
            print(f"I: {path}: found {len(raw_cases)} raw test cases")
            bi_blocks = list(
                filter(
                    None,
                    map(BasicIntegratedBlock.from_raw_block_opt, raw_cases),
                )
            )
            print(f"I: {path}: found {len(bi_blocks)} bi blocks")
            if args.vim_bin is None:
                print("I: dry-run mode")

            with DotsProgress() as progress:
                for (case_id,), fut in pmap(
                    setup_fn,
                    runner,
                    bi_blocks,
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
                    if isinstance(res, BasicIntegratedVerificationFailure):
                        print(f"F: {res}", file=sys.stderr)
                        sys.exit(1)
                    if isinstance(res, VerificationOutput):
                        json.dump(
                            {
                                "id": case_id,
                                "span": res.span,
                                "f": res.fun_name,
                                "b": res.buffer,
                                "i": res.model_input,
                                "o": res.model_output,
                            },
                            outfile,
                        )
                        outfile.write("\n")
                        written_to_unit_info = True
                    progress.step()

    if not written_to_unit_info:
        os.remove(unit_info_file)
