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

from . import basic_integrated_verification as m
from . import parser


def test_from_raw_block_opt():
    raw_test_cases = parser.RawTestCases()
    span = parser.SourceSpan.for_file("foo")
    lines = """\
#V 4

#X bi
#E CursorMoved=
#E CursorMovedI=
#E CmdlineChanged=

M n
K w
B0 |abc·def␊
C 2
S0 "a=foo

M o
K e
O y
R a
B0 [a|b]c·def␊
S1 "a= '[= ']= '<= '>=
S0 visualmode()=V

M v
K iw
B0 []abcde␊
C 0
S1 visualmode()= '[= ']=
"""
    raw_test_cases.extend_from_lines(lines.splitlines(), span)
    basic_integrated_blocks = list(
        filter(
            None,
            [
                m.BasicIntegratedBlock.from_raw_block_opt(raw_block)
                for raw_block in raw_test_cases.blocks
            ],
        )
    )
    span = parser.SourceSpan.for_file("foo")
    assert basic_integrated_blocks == [
        m.BasicIntegratedBlock(
            raw_directives=[
                parser.RawDirective("B0", "|abc·def␊", span.copy_as(10)),
                parser.RawDirective("C", "2", span.copy_as(11)),
                parser.RawDirective("E", "CursorMoved=", span.copy_as(4)),
                parser.RawDirective("E", "CursorMovedI=", span.copy_as(5)),
                parser.RawDirective("E", "CmdlineChanged=", span.copy_as(6)),
                parser.RawDirective("K", "w", span.copy_as(9)),
                parser.RawDirective("M", "n", span.copy_as(8)),
                parser.RawDirective("S0", '"a=foo', span.copy_as(12)),
                parser.RawDirective("X", "bi", span.copy_as(3)),
            ],
            span=span.copy_as(8, 12),
            mode="n",
            motion_key="w",
            count="2",
            operator=None,
            register=None,
            initial_visualmode=None,
            initial_visual_begin=None,
            initial_visual_end=None,
            initial_cursor=[0, 1, 1, 0, 1],
            initial_states=[parser.StateExpr("reg", "a", "foo")],
            states_to_verify=[],
            autocmd_events_to_verify=[
                "CursorMoved",
                "CursorMovedI",
                "CmdlineChanged",
            ],
        ),
        m.BasicIntegratedBlock(
            raw_directives=[
                parser.RawDirective("B0", "[a|b]c·def␊", span.copy_as(18)),
                parser.RawDirective("E", "CursorMoved=", span.copy_as(4)),
                parser.RawDirective("E", "CursorMovedI=", span.copy_as(5)),
                parser.RawDirective("E", "CmdlineChanged=", span.copy_as(6)),
                parser.RawDirective("K", "e", span.copy_as(15)),
                parser.RawDirective("M", "o", span.copy_as(14)),
                parser.RawDirective("O", "y", span.copy_as(16)),
                parser.RawDirective("R", "a", span.copy_as(17)),
                parser.RawDirective("S0", "visualmode()=V", span.copy_as(20)),
                parser.RawDirective("S1", '"a=', span.copy_as(19)),
                parser.RawDirective("S1", "'[=", span.copy_as(19)),
                parser.RawDirective("S1", "']=", span.copy_as(19)),
                parser.RawDirective("S1", "'<=", span.copy_as(19)),
                parser.RawDirective("S1", "'>=", span.copy_as(19)),
                parser.RawDirective("X", "bi", span.copy_as(3)),
            ],
            span=span.copy_as(14, 20),
            mode="o",
            motion_key="e",
            count="",
            operator="y",
            register="a",
            initial_visualmode="V",
            initial_visual_begin=[0, 1, 1, 0],
            initial_visual_end=[0, 1, 3, 0],
            initial_cursor=[0, 1, 2, 0, 2],
            initial_states=[parser.StateExpr("func", "visualmode", "V")],
            states_to_verify=[
                parser.StateExpr("reg", "a", None),
                parser.StateExpr("mark", "[", None),
                parser.StateExpr("mark", "]", None),
                parser.StateExpr("mark", "<", None),
                parser.StateExpr("mark", ">", None),
            ],
            autocmd_events_to_verify=[
                "CursorMoved",
                "CursorMovedI",
                "CmdlineChanged",
            ],
        ),
        m.BasicIntegratedBlock(
            raw_directives=[
                parser.RawDirective("B0", "[]abcde␊", span.copy_as(24)),
                parser.RawDirective("C", "0", span.copy_as(25)),
                parser.RawDirective("E", "CursorMoved=", span.copy_as(4)),
                parser.RawDirective("E", "CursorMovedI=", span.copy_as(5)),
                parser.RawDirective("E", "CmdlineChanged=", span.copy_as(6)),
                parser.RawDirective("K", "iw", span.copy_as(23)),
                parser.RawDirective("M", "v", span.copy_as(22)),
                parser.RawDirective("S1", "visualmode()=", span.copy_as(26)),
                parser.RawDirective("S1", "'[=", span.copy_as(26)),
                parser.RawDirective("S1", "']=", span.copy_as(26)),
                parser.RawDirective("X", "bi", span.copy_as(3)),
            ],
            span=span.copy_as(22, 26),
            mode="x",
            motion_key="iw",
            count="0",
            operator=None,
            register=None,
            initial_visualmode="v",
            initial_visual_begin=[0, 1, 1, 0],
            initial_visual_end=[0, 1, 1, 0],
            initial_cursor=None,
            initial_states=[],
            states_to_verify=[
                parser.StateExpr("func", "visualmode", None),
                parser.StateExpr("mark", "[", None),
                parser.StateExpr("mark", "]", None),
            ],
            autocmd_events_to_verify=[
                "CursorMoved",
                "CursorMovedI",
                "CmdlineChanged",
            ],
        ),
    ]
