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

from dataclasses import dataclass
import string
from typing import Literal

from .parser import (
    RawBlock,
    RawDirective,
    SourceSpan,
    StateExpr,
    ParseError,
    BufferExpr,
    AutocmdEventCountExpr,
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


@dataclass
class BasicIntegratedBlock:
    raw_directives: list[RawDirective]
    # Block-level span.
    span: SourceSpan

    mode: Literal["n", "x", "o"]
    motion_key: str
    # Either a positive integer as string or empty.
    count: str
    # If mode is not "o", this will be None.
    operator: str | None
    # If mode is not "o", this will be None. When this is not None, an empty
    # string value denotes the default implicit register.
    register: str | None

    initial_visualmode: Literal["v", "V", "\\<C-v>"] | None
    initial_visual_begin: list[int] | None
    initial_visual_end: list[int] | None

    # If mode is "x", this may be None.
    initial_cursor: list[int] | None

    # States before to setup and check.
    initial_states: list[StateExpr]
    # States to verify after the motion.
    states_to_verify: list[StateExpr]
    # Autocmd event counts to verify after the motion.
    autocmd_events_to_verify: list[str]

    @classmethod
    def from_raw_block_opt(cls, raw_block: RawBlock):
        """
        Return None if `raw_block` is not declared to export to basic
        integrated verification block; else, construct a new basic integrated
        verification block from the raw block.
        """
        if all(dr.arg != "bi" for dr in raw_block.iter_directives_like("X")):
            return None

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
            raw_directives=raw_block.directives,
            span=raw_block.span,
            mode=mode,
            motion_key=motion_key,
            count=count,
            operator=operator,
            register=register,
            initial_visualmode=initial_visualmode,
            initial_visual_begin=initial_visual_begin,
            initial_visual_end=initial_visual_end,
            initial_cursor=initial_cursor,
            initial_states=initial_states,
            states_to_verify=states_to_verify,
            autocmd_events_to_verify=autocmd_events_to_verify,
        )
