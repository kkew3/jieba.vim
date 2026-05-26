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

"""Basic parsing."""

from dataclasses import dataclass
from typing import Iterable, Literal


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

        for lineno, line in enumerate(lines, 1):
            line = line.rstrip("\n")
            # Skip comments.
            if line.startswith("//"):
                continue
            # Reset defaults.
            if line.startswith("##"):
                defaults.clear()
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
