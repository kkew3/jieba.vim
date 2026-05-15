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
from typing import Iterable, Iterator, Literal


@dataclass
class SourceSpan:
    file: str | None = None
    lineno: int = 0
    lineno_end: int = 0

    @classmethod
    def for_file(cls, file: str) -> "SourceSpan":
        return cls(file)

    def __str__(self):
        file = self.file or "_"
        if self.lineno and self.lineno_end:
            s = f"{file}:{self.lineno}-{self.lineno_end}"
        elif self.lineno:
            s = f"{file}:{self.lineno}"
        else:
            s = f"{file}"
        return s

    def to_parse_error(self, reason: str) -> "ParseError":
        return ParseError(self, reason)

    def copy(self) -> "SourceSpan":
        return SourceSpan(self.file, self.lineno, self.lineno_end)

    def copy_as(self, lineno: int = 0, lineno_end: int = 0) -> "SourceSpan":
        return SourceSpan(self.file, lineno, lineno_end)


class ParseError(Exception):
    def __init__(self, span: SourceSpan, reason: str):
        super().__init__(f"parsing error: {span}: {reason}")


@dataclass
class RawDirective:
    ty: str
    arg: str
    # Directive-level span.
    span: SourceSpan


@dataclass
class RawBlock:
    directives: list[RawDirective]
    # Block-level span.
    span: SourceSpan

    def __init__(self, directives: list[RawDirective]) -> None:
        self.directives = directives
        self.span = SourceSpan()
        if self.directives:
            self.span = self.directives[0].span.copy()
            self.span.lineno_end = self.span.lineno
            for dr in self.directives[1:]:
                self.span.lineno = min(self.span.lineno, dr.span.lineno)
                self.span.lineno_end = max(self.span.lineno_end, dr.span.lineno)
        self.sort_by_dtype()

    def extend_globals(self, global_directives: list[RawDirective]) -> None:
        """
        Extend `self` with global directives without updating the block source
        span.

        Raise ValueError:

        - If `directives` contains any directive whose type already exists in
          self block.
        """
        if any(self.contains_directive_like(dr) for dr in global_directives):
            raise ValueError(
                f"globals `{global_directives}` already exists in block: {self}"
            )
        self.directives.extend(global_directives)
        self.sort_by_dtype()

    def extend_defaults(self, default_directives: list[RawDirective]) -> None:
        """
        Extend `self` with localized default directives without updating the
        block source span. Only those default directives whose types do not yet
        exist in `self` get appended.
        """
        absent_defaults = [
            dr
            for dr in default_directives
            if not self.contains_directive_like(dr)
        ]
        self.directives.extend(absent_defaults)
        self.sort_by_dtype()

    def contains_directive_like(self, dr: RawDirective | str) -> bool:
        """
        Return true if self block contains any directive whose type is the same
        as the type of `dr`.
        """
        dr_type = dr if isinstance(dr, str) else dr.ty
        return any(dr.ty == dr_type for dr in self.directives)

    def sort_by_dtype(self):
        """Stable sort `self` by directive types."""
        self.directives.sort(key=lambda dr: dr.ty)

    def iter_directives_like(
        self, dr: RawDirective | str
    ) -> Iterator[RawDirective]:
        dr_type = dr if isinstance(dr, str) else dr.ty
        return iter(dr for dr in self.directives if dr.ty == dr_type)


@dataclass
class RawTestCases:
    blocks: list[RawBlock]

    def __init__(self):
        self.blocks = []

    def extend_from_lines(self, lines: Iterable[str], span: SourceSpan):
        # Required version.
        ver: list[RawDirective] = []
        # Head conditionals.
        hc: list[RawDirective] = []
        # Defaults.
        defaults: list[RawDirective] = []
        # Directives to collect as current block.
        current_block: list[RawDirective] = []
        # Parsing stages.
        stage: Literal["PRE_BLOCK", "INSIDE_BLOCK", "OUTSIDE_BLOCK"] = (
            "PRE_BLOCK"
        )
        supported_defaults = {
            "X",
            "M",
            "K",
            "O",
            "R",
            "C",
            "S0",
            "B0",
            "Q",
            "E",
        }

        for lineno, line in enumerate(lines, 1):
            line = line.rstrip("\n")
            # Skip comments.
            if line.startswith("//"):
                continue
            # Reset defaults.
            if line.startswith("##"):
                defaults.clear()
                continue
            line_span = span.copy_as(lineno)

            if stage == "PRE_BLOCK":
                # Accept blank line and any directives. Transit to INSIDE_BLOCK
                # on local directive.
                splits = line.split()
                if not splits:
                    continue

                # If line is not blank ..
                ty, *args = splits
                if ty == "?":
                    # Line is head conditional.
                    hc.extend(
                        RawDirective(ty, a, line_span.copy()) for a in args
                    )
                elif ty == "#V":
                    # Line is a global required version.
                    ver.extend(
                        RawDirective("V", a, line_span.copy()) for a in args
                    )
                elif ty.startswith("#"):
                    # Line is a global default.
                    if ty[1:] not in supported_defaults:
                        raise line_span.to_parse_error(
                            f"unsupported directive `{ty}`"
                        )
                    defaults.extend(
                        RawDirective(ty[1:], a, line_span.copy()) for a in args
                    )
                else:
                    # Line is a local directive.
                    current_block.extend(
                        RawDirective(ty, a, line_span.copy()) for a in args
                    )
                    stage = "INSIDE_BLOCK"
            elif stage == "INSIDE_BLOCK":
                # Accept blank line and local directives. On blank line, finish
                # current block and transit to OUTSIDE_BLOCK.
                splits = line.split()
                if not splits:
                    # Line is blank.
                    new_raw_block = RawBlock(current_block)
                    new_raw_block.extend_defaults(defaults)
                    new_raw_block.extend_globals(ver)
                    new_raw_block.extend_globals(hc)
                    self.blocks.append(new_raw_block)
                    current_block = []
                    stage = "OUTSIDE_BLOCK"
                    continue

                # If line is not blank ..
                ty, *args = splits
                if ty == "?" or ty.startswith("#"):
                    raise line_span.to_parse_error(
                        f"invalid token at line: {line}"
                    )

                # If line is a local directive ..
                current_block.extend(
                    RawDirective(ty, a, line_span.copy()) for a in args
                )
            else:
                # Accept blank line, global defaults and local directives but
                # does not accept head conditionals or global required version.
                # On local directive, transit to INSIDE_BLOCK.
                splits = line.split()
                if not splits:
                    continue

                # If line is not blank ..
                ty, *args = splits
                if ty in ("?", "#V"):
                    raise line_span.to_parse_error(
                        f"invalid token at line: {line}"
                    )

                if ty.startswith("#"):
                    # If line is a global default ..
                    if ty[1:] not in supported_defaults:
                        raise line_span.to_parse_error(
                            f"unsupported directive `{ty}`"
                        )
                    defaults.extend(
                        RawDirective(ty[1:], a, line_span.copy()) for a in args
                    )
                else:
                    # If line is a local directive ..
                    current_block.extend(
                        RawDirective(ty, a, line_span.copy()) for a in args
                    )
                    stage = "INSIDE_BLOCK"

        # If there are any raw directives to collect into block ..
        if current_block:
            new_raw_block = RawBlock(current_block)
            new_raw_block.extend_defaults(defaults)
            new_raw_block.extend_globals(ver)
            new_raw_block.extend_globals(hc)
            self.blocks.append(new_raw_block)

    def extend_from_file(self, path: str):
        span = SourceSpan.for_file(path)
        with open(path, encoding="utf-8") as infile:
            self.extend_from_lines(infile, span)


@dataclass
class StateExpr:
    ty: Literal["opt", "func", "reg", "mark"]
    name: str
    # Will be of type list[int] | None when ty == "mark"; otherwise, will be of
    # type str | None.
    value: str | list[int] | None

    @classmethod
    def parse(
        cls,
        arg: str,
        span: SourceSpan,
        parse_as_incomplete: bool = False,
    ):
        """
        If `parse_as_incomplete` is passed True, the value will be ignored and
        be set to None.
        """
        name, _, value = arg.partition("=")
        if parse_as_incomplete:
            value = None
        if name.endswith("()"):
            return cls("func", name[:-2], value)
        if name.startswith('"'):
            return cls("reg", name[1:], value)
        if name.startswith("'"):
            if value is not None:
                if not value.startswith("[") or not value.endswith("]"):
                    raise span.to_parse_error(
                        f"mark's value should be list[int] but got: {value}"
                    )
                value_arr = [int(x) for x in value[1:-1].split(",")]
                return cls("mark", name[1:], value_arr)
            return cls("mark", name[1:], None)
        return cls("opt", name, value)


class Cursor:
    def __init__(self, list_):
        self.list_ = list_
        self.n = len(self.list_)
        self.j = 0

    def peek(self):
        if self.j < self.n:
            return self.list_[self.j]
        return None

    def next(self):
        if self.j < self.n:
            a = self.list_[self.j]
            self.j += 1
            return a
        return None


@dataclass
class BufferExpr:
    clean_buffer: list[str]
    # 4-element list
    langle: list[int] | None
    # 4-element list
    rangle: list[int] | None
    # 4-element list
    visual_begin: list[int] | None
    # 4-element list
    visual_end: list[int] | None
    # 5-element list
    cursor: list[int] | None

    @classmethod
    def parse(cls, arg: str, span: SourceSpan):
        position_marks = {"<", ">", "[", "]", "|", "\\"}
        eol_marks = {"␊", "␀"}
        markup_buffer = []
        line = ""
        chars = Cursor(list(arg))
        is_empty_buffer = False
        while (c := chars.next()) is not None:
            line += c
            if c in eol_marks:
                if c == "␀":
                    is_empty_buffer = True
                chars_after_lf = ""
                any_tilde = False
                while (c1 := chars.peek()) is not None and (
                    c1 == "~" or c1 in position_marks
                ):
                    chars_after_lf += chars.next()
                    if c1 == "~":
                        any_tilde = True
                if any_tilde:
                    line += chars_after_lf
                    markup_buffer.append(line)
                    line = ""
                else:
                    markup_buffer.append(line)
                    line = chars_after_lf
        if line or (is_empty_buffer and len(markup_buffer) > 1):
            raise span.to_parse_error(f"invalid directive `B_` value: {arg}")

        langle = None
        rangle = None
        visual_begin = None
        visual_end = None
        cursor = None
        clean_buffer = []
        for lineno, markup_line in enumerate(markup_buffer, 1):
            langle_col = None
            rangle_col = None
            visual_begin_col = None
            visual_end_col = None
            cursor_col = None
            curswant_col = None
            clean_chars = []
            has_virtual_cols = False
            for i, c in enumerate(reversed(markup_line)):
                if i == 0 and c == "~":
                    has_virtual_cols = True
                if c not in position_marks:
                    clean_chars.append(c)
                else:
                    if not clean_chars:
                        raise span.to_parse_error(
                            f"invalid directive `B_` value: {arg}"
                        )
                    counter = len(clean_chars)
                    if c == "<":
                        if langle is not None or langle_col is not None:
                            raise span.to_parse_error(
                                f"invalid directive `B_` value: {arg}"
                            )
                        langle_col = counter
                    elif c == ">":
                        if rangle is not None or rangle_col is not None:
                            raise span.to_parse_error(
                                f"invalid directive `B_` value: {arg}"
                            )
                        rangle_col = counter
                    elif c == "[":
                        if (
                            visual_begin is not None
                            or visual_begin_col is not None
                        ):
                            raise span.to_parse_error(
                                f"invalid directive `B_` value: {arg}"
                            )
                        visual_begin_col = counter
                    elif c == "]":
                        if visual_end is not None or visual_end_col is not None:
                            raise span.to_parse_error(
                                f"invalid directive `B_` value: {arg}"
                            )
                        visual_end_col = counter
                    elif c == "|":
                        if cursor is not None or cursor_col is not None:
                            raise span.to_parse_error(
                                f"invalid directive `B_` value: {arg}"
                            )
                        cursor_col = counter
                    elif c == "\\":
                        if cursor is not None or curswant_col is not None:
                            raise span.to_parse_error(
                                f"invalid directive `B_` value: {arg}"
                            )
                        curswant_col = counter
                    else:
                        assert False, "unreachable"
            if cursor_col is None and curswant_col is not None:
                raise span.to_parse_error(
                    f"invalid directive `B_` value: {arg}"
                )

            implicit_curswant = False
            if cursor_col is not None and curswant_col is None:
                curswant_col = cursor_col
                implicit_curswant = True

            # Take complementation.
            n_clean = len(clean_chars)
            if langle_col is not None:
                langle_col = n_clean - langle_col
            if rangle_col is not None:
                rangle_col = n_clean - rangle_col
            if visual_begin_col is not None:
                visual_begin_col = n_clean - visual_begin_col
            if visual_end_col is not None:
                visual_end_col = n_clean - visual_end_col
            if cursor_col is not None:
                cursor_col = n_clean - cursor_col
            if curswant_col is not None:
                curswant_col = n_clean - curswant_col

            # Compute (col, off) and prune auxiliary chars.
            col = 0
            off = 0
            curswant = 0
            cursor_curswant = None
            final_clean_chars = []
            cursor_assigned = False
            for i, c in enumerate(reversed(clean_chars)):
                if col == 0 and c == "~":
                    raise span.to_parse_error(
                        f"invalid directive `B_` value: {arg}"
                    )
                if c == "~":
                    off += 1
                else:
                    col += 1
                    off = 0
                curswant += 1

                if langle_col is not None and i == langle_col:
                    langle = [0, lineno, col, off]
                if rangle_col is not None and i == rangle_col:
                    rangle = [0, lineno, col, off]
                if visual_begin_col is not None and i == visual_begin_col:
                    visual_begin = [0, lineno, col, off]
                if visual_end_col is not None and i == visual_end_col:
                    visual_end = [0, lineno, col, off]
                if cursor_col is not None and i == cursor_col:
                    cursor = [0, lineno, col, off, 0]
                    cursor_assigned = True
                if curswant_col is not None and i == curswant_col:
                    if (
                        c in eol_marks
                        and not implicit_curswant
                        and not has_virtual_cols
                    ):
                        cursor_curswant = 2147483647  # v:maxcol
                    else:
                        cursor_curswant = curswant

                if c == "·":
                    final_clean_chars.append(" ")
                elif c == "┤":
                    final_clean_chars.append("\t")
                elif c not in eol_marks and c not in {"@", "~"}:
                    final_clean_chars.append(c)

            if cursor_assigned and cursor is not None:
                assert cursor_curswant is not None
                cursor[4] = cursor_curswant

            clean_buffer.append("".join(final_clean_chars))

        if is_empty_buffer:
            if any(s for s in clean_buffer):
                raise span.to_parse_error(
                    f"invalid directive `B_` value: {arg}"
                )
            clean_buffer.clear()

        return cls(
            clean_buffer,
            langle,
            rangle,
            visual_begin,
            visual_end,
            cursor,
        )


@dataclass
class AutocmdEventCountExpr:
    name: str
    count: int | None

    @classmethod
    def parse(
        cls,
        arg: str,
        span: SourceSpan,
        parse_as_incomplete: bool = False,
    ):
        """
        If `parse_as_incomplete` is passed True, the count will be ignored and
        be set to None.
        """
        name, _, value = arg.partition("=")
        if not name:
            raise span.to_parse_error(f"invalid directive `E` value: {arg}")
        if parse_as_incomplete:
            count = None
        else:
            try:
                count = int(value)
            except ValueError as err:
                raise span.to_parse_error(
                    f"invalid directive `E` value: {arg}"
                ) from err
        return cls(name, count)
