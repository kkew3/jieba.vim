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

from io import StringIO

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
            raw_directives=tuple(
                [
                    parser.RawDirective("B0", "|abc·def␊", span.copy_as(10)),
                    parser.RawDirective("C", "2", span.copy_as(11)),
                    parser.RawDirective("E", "CursorMoved=", span.copy_as(4)),
                    parser.RawDirective("E", "CursorMovedI=", span.copy_as(5)),
                    parser.RawDirective(
                        "E", "CmdlineChanged=", span.copy_as(6)
                    ),
                    parser.RawDirective("K", "w", span.copy_as(9)),
                    parser.RawDirective("M", "n", span.copy_as(8)),
                    parser.RawDirective("S0", '"a=foo', span.copy_as(12)),
                    parser.RawDirective("X", "bi", span.copy_as(3)),
                ]
            ),
            span=span.copy_as(8, 12),
            error_suppressed=False,
            hc=(),
            mode="n",
            motion_key="w",
            count="2",
            operator=None,
            register=None,
            clean_buffer_before=tuple(["abc def"]),
            initial_visualmode=None,
            initial_visual_begin=None,
            initial_visual_end=None,
            initial_cursor=tuple([0, 1, 1, 0, 1]),
            initial_states=tuple([parser.StateExpr("reg", "a", "foo")]),
            states_to_verify=(),
            autocmd_events_to_verify=tuple(
                [
                    "CursorMoved",
                    "CursorMovedI",
                    "CmdlineChanged",
                ]
            ),
        ),
        m.BasicIntegratedBlock(
            raw_directives=tuple(
                [
                    parser.RawDirective("B0", "[a|b]c·def␊", span.copy_as(18)),
                    parser.RawDirective("E", "CursorMoved=", span.copy_as(4)),
                    parser.RawDirective("E", "CursorMovedI=", span.copy_as(5)),
                    parser.RawDirective(
                        "E", "CmdlineChanged=", span.copy_as(6)
                    ),
                    parser.RawDirective("K", "e", span.copy_as(15)),
                    parser.RawDirective("M", "o", span.copy_as(14)),
                    parser.RawDirective("O", "y", span.copy_as(16)),
                    parser.RawDirective("R", "a", span.copy_as(17)),
                    parser.RawDirective(
                        "S0", "visualmode()=V", span.copy_as(20)
                    ),
                    parser.RawDirective("S1", '"a=', span.copy_as(19)),
                    parser.RawDirective("S1", "'[=", span.copy_as(19)),
                    parser.RawDirective("S1", "']=", span.copy_as(19)),
                    parser.RawDirective("S1", "'<=", span.copy_as(19)),
                    parser.RawDirective("S1", "'>=", span.copy_as(19)),
                    parser.RawDirective("X", "bi", span.copy_as(3)),
                ]
            ),
            span=span.copy_as(14, 20),
            error_suppressed=False,
            hc=(),
            mode="o",
            motion_key="e",
            count="",
            operator="y",
            register="a",
            clean_buffer_before=tuple(["abc def"]),
            initial_visualmode="V",
            initial_visual_begin=tuple([0, 1, 1, 0]),
            initial_visual_end=tuple([0, 1, 3, 0]),
            initial_cursor=tuple([0, 1, 2, 0, 2]),
            initial_states=tuple([parser.StateExpr("func", "visualmode", "V")]),
            states_to_verify=tuple(
                [
                    parser.StateExpr("reg", "a", None),
                    parser.StateExpr("mark", "[", None),
                    parser.StateExpr("mark", "]", None),
                    parser.StateExpr("mark", "<", None),
                    parser.StateExpr("mark", ">", None),
                ]
            ),
            autocmd_events_to_verify=tuple(
                [
                    "CursorMoved",
                    "CursorMovedI",
                    "CmdlineChanged",
                ]
            ),
        ),
        m.BasicIntegratedBlock(
            raw_directives=tuple(
                [
                    parser.RawDirective("B0", "[]abcde␊", span.copy_as(24)),
                    parser.RawDirective("C", "0", span.copy_as(25)),
                    parser.RawDirective("E", "CursorMoved=", span.copy_as(4)),
                    parser.RawDirective("E", "CursorMovedI=", span.copy_as(5)),
                    parser.RawDirective(
                        "E", "CmdlineChanged=", span.copy_as(6)
                    ),
                    parser.RawDirective("K", "iw", span.copy_as(23)),
                    parser.RawDirective("M", "v", span.copy_as(22)),
                    parser.RawDirective(
                        "S1", "visualmode()=", span.copy_as(26)
                    ),
                    parser.RawDirective("S1", "'[=", span.copy_as(26)),
                    parser.RawDirective("S1", "']=", span.copy_as(26)),
                    parser.RawDirective("X", "bi", span.copy_as(3)),
                ]
            ),
            span=span.copy_as(22, 26),
            error_suppressed=False,
            hc=(),
            mode="x",
            motion_key="iw",
            count="",
            operator=None,
            register=None,
            clean_buffer_before=tuple(["abcde"]),
            initial_visualmode="v",
            initial_visual_begin=tuple([0, 1, 1, 0]),
            initial_visual_end=tuple([0, 1, 1, 0]),
            initial_cursor=None,
            initial_states=(),
            states_to_verify=tuple(
                [
                    parser.StateExpr("func", "visualmode", None),
                    parser.StateExpr("mark", "[", None),
                    parser.StateExpr("mark", "]", None),
                ]
            ),
            autocmd_events_to_verify=tuple(
                [
                    "CursorMoved",
                    "CursorMovedI",
                    "CmdlineChanged",
                ]
            ),
        ),
    ]


def test_write_std_run_custom_run():
    lines = """\
#V 4
? !has:nvim

#X bi

M n
S0 visualmode()=v
S0 selection=exclusive
K w
B0 |ab[]c·def␊
S1 visualmode()=
E CursorMoved=
E CmdlineChanged=

M \\<C-v>
S0 virtualedit=onemore "a=foo
K e
B0 []abc·def␊
C 1
E CursorMoved=
E CmdlineChanged=
S1 visualmode()= '[= ']=

M o
K W
O d
R a
B0 a|bc·def␊
C 2
S1 "a=
S1 '[=
S1 ']=
S1 '<=
S1 '>=
"""
    raw_test_cases = parser.RawTestCases()
    span = parser.SourceSpan.for_file("foo")
    raw_test_cases.extend_from_lines(lines.splitlines(), span)
    basic_integrated_blocks = [
        m.BasicIntegratedBlock.from_raw_block_opt(raw_block)
        for raw_block in raw_test_cases.blocks
    ]

    def collect_raw_directives(*tuples):
        return tuple(
            [
                parser.RawDirective(ty, arg, span.copy_as(lineno))
                for ty, arg, lineno in tuples
            ]
        )

    assert basic_integrated_blocks == [
        m.BasicIntegratedBlock(
            raw_directives=collect_raw_directives(
                ("?", "!has:nvim", 2),
                ("B0", "|ab[]c·def␊", 10),
                ("E", "CursorMoved=", 12),
                ("E", "CmdlineChanged=", 13),
                ("K", "w", 9),
                ("M", "n", 6),
                ("S0", "visualmode()=v", 7),
                ("S0", "selection=exclusive", 8),
                ("S1", "visualmode()=", 11),
                ("X", "bi", 4),
            ),
            span=span.copy_as(6, 13),
            error_suppressed=False,
            hc=tuple(
                [
                    parser.HeadConditionalExpr("non_feature", "nvim"),
                ]
            ),
            mode="n",
            motion_key="w",
            count="",
            operator=None,
            register=None,
            clean_buffer_before=tuple(["abc def"]),
            initial_visualmode="v",
            initial_visual_begin=tuple([0, 1, 3, 0]),
            initial_visual_end=tuple([0, 1, 3, 0]),
            initial_cursor=tuple([0, 1, 1, 0, 1]),
            initial_states=tuple(
                [
                    parser.StateExpr("func", "visualmode", "v"),
                    parser.StateExpr("opt", "selection", "exclusive"),
                ]
            ),
            states_to_verify=tuple(
                [
                    parser.StateExpr("func", "visualmode", None),
                ]
            ),
            autocmd_events_to_verify=tuple(
                [
                    "CursorMoved",
                    "CmdlineChanged",
                ]
            ),
        ),
        m.BasicIntegratedBlock(
            raw_directives=collect_raw_directives(
                ("?", "!has:nvim", 2),
                ("B0", "[]abc·def␊", 18),
                ("C", "1", 19),
                ("E", "CursorMoved=", 20),
                ("E", "CmdlineChanged=", 21),
                ("K", "e", 17),
                ("M", "\\<C-v>", 15),
                ("S0", "virtualedit=onemore", 16),
                ("S0", '"a=foo', 16),
                ("S1", "visualmode()=", 22),
                ("S1", "'[=", 22),
                ("S1", "']=", 22),
                ("X", "bi", 4),
            ),
            span=span.copy_as(15, 22),
            error_suppressed=False,
            hc=tuple(
                [
                    parser.HeadConditionalExpr("non_feature", "nvim"),
                ]
            ),
            mode="x",
            motion_key="e",
            count="1",
            operator=None,
            register=None,
            clean_buffer_before=tuple(["abc def"]),
            initial_visualmode="\\<C-v>",
            initial_visual_begin=tuple([0, 1, 1, 0]),
            initial_visual_end=tuple([0, 1, 1, 0]),
            initial_cursor=None,
            initial_states=tuple(
                [
                    parser.StateExpr("opt", "virtualedit", "onemore"),
                    parser.StateExpr("reg", "a", "foo"),
                ]
            ),
            states_to_verify=tuple(
                [
                    parser.StateExpr("func", "visualmode", None),
                    parser.StateExpr("mark", "[", None),
                    parser.StateExpr("mark", "]", None),
                ]
            ),
            autocmd_events_to_verify=tuple(
                [
                    "CursorMoved",
                    "CmdlineChanged",
                ]
            ),
        ),
        m.BasicIntegratedBlock(
            raw_directives=collect_raw_directives(
                ("?", "!has:nvim", 2),
                ("B0", "a|bc·def␊", 28),
                ("C", "2", 29),
                ("K", "W", 25),
                ("M", "o", 24),
                ("O", "d", 26),
                ("R", "a", 27),
                ("S1", '"a=', 30),
                ("S1", "'[=", 31),
                ("S1", "']=", 32),
                ("S1", "'<=", 33),
                ("S1", "'>=", 34),
                ("X", "bi", 4),
            ),
            span=span.copy_as(24, 34),
            error_suppressed=False,
            hc=tuple(
                [
                    parser.HeadConditionalExpr("non_feature", "nvim"),
                ]
            ),
            mode="o",
            motion_key="W",
            count="2",
            operator="d",
            register="a",
            clean_buffer_before=tuple(["abc def"]),
            initial_visualmode=None,
            initial_visual_begin=None,
            initial_visual_end=None,
            initial_cursor=tuple([0, 1, 2, 0, 2]),
            initial_states=(),
            states_to_verify=tuple(
                [
                    parser.StateExpr("reg", "a", None),
                    parser.StateExpr("mark", "[", None),
                    parser.StateExpr("mark", "]", None),
                    parser.StateExpr("mark", "<", None),
                    parser.StateExpr("mark", ">", None),
                ]
            ),
            autocmd_events_to_verify=(),
        ),
    ]

    sbuf = StringIO()
    basic_integrated_blocks[0].write_std_run(sbuf)
    assert (
        sbuf.getvalue()
        == """\
if has("nvim")
    if has("nvim")
        lua <<EOF
io.write(vim.fn.json_encode({cf = "continue"}) .. "\\n")
EOF
    else
        execute "!echo " . shellescape(escape(json_encode({"cf": "continue"}), "\\\\"), 1)
    endif
endif

" define oracle model
function! JiebaOracleModel(...)
    let g:model_input = a:000
    let g:model_output = call(function("JiebaModelNmap"), a:000)
    return g:model_output
endfunction

" state_before setup
let &selection = "exclusive"

" buffer_before setup
call setpos(".", [0, 1, 3, 0])
execute "normal! v\\<Esc>"
call setpos("'>", [0, 1, 3, 0])
call setpos(".", [0, 1, 1, 0, 1])

" autocmd setup
function! IncrementAutocmdEventCount(event_name)
    let l:count = get(g:jieba_test_case_events_count, a:event_name, 0)
    let g:jieba_test_case_events_count[a:event_name] = l:count + 1
endfunction

augroup jieba_test_case_autocmd_events_monitoring
    autocmd!
    au CursorMoved * call IncrementAutocmdEventCount("CursorMoved")
    au CmdlineChanged * call IncrementAutocmdEventCount("CmdlineChanged")
augroup END

" state_before checking
if visualmode() !=# "v"
    if has("nvim")
        lua <<EOF
io.stderr:write("unexpected state_before in function visualmode()" .. " actual:: " .. vim.fn.json_encode(vim.fn.visualmode()) .. " expected:: " .. vim.fn.json_encode("v") .. "\\n")
EOF
    else
        execute "!echo " . shellescape("unexpected state_before in function visualmode()" . " actual:: " . escape(json_encode(visualmode()), "\\\\") . " expected:: " . escape(json_encode("v"), "\\\\"), 1) . " >&2"
    endif
    cquit
    finish
endif
if &selection !=# "exclusive"
    if has("nvim")
        lua <<EOF
io.stderr:write("unexpected state_before in option 'selection'" .. " actual:: " .. vim.fn.json_encode(vim.o.selection) .. " expected:: " .. vim.fn.json_encode("exclusive") .. "\\n")
EOF
    else
        execute "!echo " . shellescape("unexpected state_before in option 'selection'" . " actual:: " . escape(json_encode(&selection), "\\\\") . " expected:: " . escape(json_encode("exclusive"), "\\\\"), 1) . " >&2"
    endif
    cquit
    finish
endif


let g:jieba_test_case_events_count = {}
" cursor movement
normal! w
execute "normal! \\<Esc>"

let s:jieba_test_case_events_count_frozen = copy(g:jieba_test_case_events_count)

" autocmd event counts querying
let g:JiebaTestGroundtruthAutocmdEventsCount = json_encode(s:jieba_test_case_events_count_frozen)

" state_after querying
let g:JiebaTestGroundtruthFunc_visualmode = visualmode()

" buffer_after querying
let g:JiebaTestGroundtruthCursor = json_encode(getcurpos())

execute "mksession! " . expand("%:p:h") . "/Session.vim"
silent xit
"""
    )
    sbuf = StringIO()
    basic_integrated_blocks[0].write_custom_run(sbuf)
    assert (
        sbuf.getvalue()
        == """\
if has("nvim")
    if has("nvim")
        lua <<EOF
io.write(vim.fn.json_encode({cf = "continue"}) .. "\\n")
EOF
    else
        execute "!echo " . shellescape(escape(json_encode({"cf": "continue"}), "\\\\"), 1)
    endif
endif

silent execute "source " . expand("%:p:h") . "/Session.vim"

" define oracle model
function! JiebaOracleModel(...)
    let g:model_input = a:000
    let g:model_output = call(function("JiebaModelNmap"), a:000)
    return g:model_output
endfunction

" state_before setup
let &selection = "exclusive"

" buffer_before setup
call setpos(".", [0, 1, 3, 0])
execute "normal! v\\<Esc>"
call setpos("'>", [0, 1, 3, 0])
call setpos(".", [0, 1, 1, 0, 1])

" autocmd setup
function! IncrementAutocmdEventCount(event_name)
    let l:count = get(g:jieba_test_case_events_count, a:event_name, 0)
    let g:jieba_test_case_events_count[a:event_name] = l:count + 1
endfunction

augroup jieba_test_case_autocmd_events_monitoring
    autocmd!
    au CursorMoved * call IncrementAutocmdEventCount("CursorMoved")
    au CmdlineChanged * call IncrementAutocmdEventCount("CmdlineChanged")
augroup END

" state_before checking
if visualmode() !=# "v"
    if has("nvim")
        lua <<EOF
io.stderr:write("unexpected state_before in function visualmode()" .. " actual:: " .. vim.fn.json_encode(vim.fn.visualmode()) .. " expected:: " .. vim.fn.json_encode("v") .. "\\n")
EOF
    else
        execute "!echo " . shellescape("unexpected state_before in function visualmode()" . " actual:: " . escape(json_encode(visualmode()), "\\\\") . " expected:: " . escape(json_encode("v"), "\\\\"), 1) . " >&2"
    endif
    cquit
    finish
endif
if &selection !=# "exclusive"
    if has("nvim")
        lua <<EOF
io.stderr:write("unexpected state_before in option 'selection'" .. " actual:: " .. vim.fn.json_encode(vim.o.selection) .. " expected:: " .. vim.fn.json_encode("exclusive") .. "\\n")
EOF
    else
        execute "!echo " . shellescape("unexpected state_before in option 'selection'" . " actual:: " . escape(json_encode(&selection), "\\\\") . " expected:: " . escape(json_encode("exclusive"), "\\\\"), 1) . " >&2"
    endif
    cquit
    finish
endif


let g:jieba_test_case_events_count = {}
" cursor movement
call JiebaNmap("w", 0, "JiebaOracleModel")
execute "normal! \\<Esc>"

let g:jieba_test_case_events_count_frozen = copy(g:jieba_test_case_events_count)

" autocmd event counts checking
if g:jieba_test_case_events_count_frozen !=# json_decode(g:JiebaTestGroundtruthAutocmdEventsCount)
    if has("nvim")
        lua <<EOF
io.stderr:write("unexpected autocmd events count" .. " actual:: " .. vim.fn.json_encode(vim.g.jieba_test_case_events_count_frozen) .. " expected:: " .. vim.fn.json_encode(vim.fn.json_decode(vim.g.JiebaTestGroundtruthAutocmdEventsCount)) .. "\\n")
EOF
    else
        execute "!echo " . shellescape("unexpected autocmd events count" . " actual:: " . escape(json_encode(g:jieba_test_case_events_count_frozen), "\\\\") . " expected:: " . escape(json_encode(json_decode(g:JiebaTestGroundtruthAutocmdEventsCount)), "\\\\"), 1) . " >&2"
    endif
    cquit
    finish
endif

" state_after checking
if visualmode() !=# g:JiebaTestGroundtruthFunc_visualmode
    if has("nvim")
        lua <<EOF
io.stderr:write("unexpected state_after in function visualmode()" .. " actual:: " .. vim.fn.json_encode(vim.fn.visualmode()) .. " expected:: " .. vim.fn.json_encode(vim.g.JiebaTestGroundtruthFunc_visualmode) .. "\\n")
EOF
    else
        execute "!echo " . shellescape("unexpected state_after in function visualmode()" . " actual:: " . escape(json_encode(visualmode()), "\\\\") . " expected:: " . escape(json_encode(g:JiebaTestGroundtruthFunc_visualmode), "\\\\"), 1) . " >&2"
    endif
    cquit
    finish
endif

" buffer_after checking
if getcurpos() !=# json_decode(g:JiebaTestGroundtruthCursor)
    if has("nvim")
        lua <<EOF
io.stderr:write("unexpected cursor position in buffer_after" .. " actual:: " .. vim.fn.json_encode(vim.fn.getcurpos()) .. " expected:: " .. vim.fn.json_encode(vim.fn.json_decode(vim.g.JiebaTestGroundtruthCursor)) .. "\\n")
EOF
    else
        execute "!echo " . shellescape("unexpected cursor position in buffer_after" . " actual:: " . escape(json_encode(getcurpos()), "\\\\") . " expected:: " . escape(json_encode(json_decode(g:JiebaTestGroundtruthCursor)), "\\\\"), 1) . " >&2"
    endif
    cquit
    finish
endif

" model_output echoing
if has("nvim")
    lua <<EOF
io.write(vim.fn.json_encode({i = vim.g.model_input, o = vim.g.model_output}) .. "\\n")
EOF
else
    execute "!echo " . shellescape(escape(json_encode({"i": g:model_input, "o": g:model_output}), "\\\\"), 1)
endif

silent xit
"""
    )

    sbuf = StringIO()
    basic_integrated_blocks[1].write_std_run(sbuf)
    assert (
        sbuf.getvalue()
        == """\
if has("nvim")
    if has("nvim")
        lua <<EOF
io.write(vim.fn.json_encode({cf = "continue"}) .. "\\n")
EOF
    else
        execute "!echo " . shellescape(escape(json_encode({"cf": "continue"}), "\\\\"), 1)
    endif
endif

" define oracle model
function! JiebaOracleModel(...)
    let g:model_input = a:000
    let g:model_output = call(function("JiebaModelXmap"), a:000)
    return g:model_output
endfunction

" state_before setup
let &virtualedit = "onemore"
call setreg("a", "foo")

" buffer_before setup
call setpos(".", [0, 1, 1, 0])
execute "normal! \\<C-v>\\<Esc>"
call setpos("'>", [0, 1, 1, 0])

" autocmd setup
function! IncrementAutocmdEventCount(event_name)
    let l:count = get(g:jieba_test_case_events_count, a:event_name, 0)
    let g:jieba_test_case_events_count[a:event_name] = l:count + 1
endfunction

augroup jieba_test_case_autocmd_events_monitoring
    autocmd!
    au CursorMoved * call IncrementAutocmdEventCount("CursorMoved")
    au CmdlineChanged * call IncrementAutocmdEventCount("CmdlineChanged")
augroup END

" state_before checking
if &virtualedit !=# "onemore"
    if has("nvim")
        lua <<EOF
io.stderr:write("unexpected state_before in option 'virtualedit'" .. " actual:: " .. vim.fn.json_encode(vim.o.virtualedit) .. " expected:: " .. vim.fn.json_encode("onemore") .. "\\n")
EOF
    else
        execute "!echo " . shellescape("unexpected state_before in option 'virtualedit'" . " actual:: " . escape(json_encode(&virtualedit), "\\\\") . " expected:: " . escape(json_encode("onemore"), "\\\\"), 1) . " >&2"
    endif
    cquit
    finish
endif
if getreg("a") !=# "foo"
    if has("nvim")
        lua <<EOF
io.stderr:write("unexpected state_before in register \\"a" .. " actual:: " .. vim.fn.json_encode(vim.fn.getreg("a")) .. " expected:: " .. vim.fn.json_encode("foo") .. "\\n")
EOF
    else
        execute "!echo " . shellescape("unexpected state_before in register \\"a" . " actual:: " . escape(json_encode(getreg("a")), "\\\\") . " expected:: " . escape(json_encode("foo"), "\\\\"), 1) . " >&2"
    endif
    cquit
    finish
endif


let g:jieba_test_case_events_count = {}
" cursor movement
normal! gv1e
execute "normal! \\<Esc>"

let s:jieba_test_case_events_count_frozen = copy(g:jieba_test_case_events_count)

" autocmd event counts querying
let g:JiebaTestGroundtruthAutocmdEventsCount = json_encode(s:jieba_test_case_events_count_frozen)

" state_after querying
let g:JiebaTestGroundtruthFunc_visualmode = visualmode()
let g:JiebaTestGroundtruthMark_lsquare = json_encode(getpos("'["))
let g:JiebaTestGroundtruthMark_rsquare = json_encode(getpos("']"))

" buffer_after querying
let g:JiebaTestGroundtruthCursor = json_encode(getcurpos())
normal! gvomaomb
let g:JiebaTestGroundtruthVisualBegin = json_encode(getpos("'a"))
let g:JiebaTestGroundtruthVisualEnd = json_encode(getpos("'b"))

execute "mksession! " . expand("%:p:h") . "/Session.vim"
silent xit
"""
    )
    sbuf = StringIO()
    basic_integrated_blocks[1].write_custom_run(sbuf)
    assert (
        sbuf.getvalue()
        == """\
if has("nvim")
    if has("nvim")
        lua <<EOF
io.write(vim.fn.json_encode({cf = "continue"}) .. "\\n")
EOF
    else
        execute "!echo " . shellescape(escape(json_encode({"cf": "continue"}), "\\\\"), 1)
    endif
endif

silent execute "source " . expand("%:p:h") . "/Session.vim"

" define oracle model
function! JiebaOracleModel(...)
    let g:model_input = a:000
    let g:model_output = call(function("JiebaModelXmap"), a:000)
    return g:model_output
endfunction

" state_before setup
let &virtualedit = "onemore"
call setreg("a", "foo")

" buffer_before setup
call setpos(".", [0, 1, 1, 0])
execute "normal! \\<C-v>\\<Esc>"
call setpos("'>", [0, 1, 1, 0])

" autocmd setup
function! IncrementAutocmdEventCount(event_name)
    let l:count = get(g:jieba_test_case_events_count, a:event_name, 0)
    let g:jieba_test_case_events_count[a:event_name] = l:count + 1
endfunction

augroup jieba_test_case_autocmd_events_monitoring
    autocmd!
    au CursorMoved * call IncrementAutocmdEventCount("CursorMoved")
    au CmdlineChanged * call IncrementAutocmdEventCount("CmdlineChanged")
augroup END

" state_before checking
if &virtualedit !=# "onemore"
    if has("nvim")
        lua <<EOF
io.stderr:write("unexpected state_before in option 'virtualedit'" .. " actual:: " .. vim.fn.json_encode(vim.o.virtualedit) .. " expected:: " .. vim.fn.json_encode("onemore") .. "\\n")
EOF
    else
        execute "!echo " . shellescape("unexpected state_before in option 'virtualedit'" . " actual:: " . escape(json_encode(&virtualedit), "\\\\") . " expected:: " . escape(json_encode("onemore"), "\\\\"), 1) . " >&2"
    endif
    cquit
    finish
endif
if getreg("a") !=# "foo"
    if has("nvim")
        lua <<EOF
io.stderr:write("unexpected state_before in register \\"a" .. " actual:: " .. vim.fn.json_encode(vim.fn.getreg("a")) .. " expected:: " .. vim.fn.json_encode("foo") .. "\\n")
EOF
    else
        execute "!echo " . shellescape("unexpected state_before in register \\"a" . " actual:: " . escape(json_encode(getreg("a")), "\\\\") . " expected:: " . escape(json_encode("foo"), "\\\\"), 1) . " >&2"
    endif
    cquit
    finish
endif


let g:jieba_test_case_events_count = {}
" cursor movement
call JiebaXmap("e", 1, "JiebaOracleModel")
execute "normal! \\<Esc>"

let g:jieba_test_case_events_count_frozen = copy(g:jieba_test_case_events_count)

" autocmd event counts checking
if g:jieba_test_case_events_count_frozen !=# json_decode(g:JiebaTestGroundtruthAutocmdEventsCount)
    if has("nvim")
        lua <<EOF
io.stderr:write("unexpected autocmd events count" .. " actual:: " .. vim.fn.json_encode(vim.g.jieba_test_case_events_count_frozen) .. " expected:: " .. vim.fn.json_encode(vim.fn.json_decode(vim.g.JiebaTestGroundtruthAutocmdEventsCount)) .. "\\n")
EOF
    else
        execute "!echo " . shellescape("unexpected autocmd events count" . " actual:: " . escape(json_encode(g:jieba_test_case_events_count_frozen), "\\\\") . " expected:: " . escape(json_encode(json_decode(g:JiebaTestGroundtruthAutocmdEventsCount)), "\\\\"), 1) . " >&2"
    endif
    cquit
    finish
endif

" state_after checking
if visualmode() !=# g:JiebaTestGroundtruthFunc_visualmode
    if has("nvim")
        lua <<EOF
io.stderr:write("unexpected state_after in function visualmode()" .. " actual:: " .. vim.fn.json_encode(vim.fn.visualmode()) .. " expected:: " .. vim.fn.json_encode(vim.g.JiebaTestGroundtruthFunc_visualmode) .. "\\n")
EOF
    else
        execute "!echo " . shellescape("unexpected state_after in function visualmode()" . " actual:: " . escape(json_encode(visualmode()), "\\\\") . " expected:: " . escape(json_encode(g:JiebaTestGroundtruthFunc_visualmode), "\\\\"), 1) . " >&2"
    endif
    cquit
    finish
endif
if getpos("'[") !=# json_decode(g:JiebaTestGroundtruthMark_lsquare)
    if has("nvim")
        lua <<EOF
io.stderr:write("unexpected state_after in mark '[" .. " actual:: " .. vim.fn.json_encode(vim.fn.getpos("'[")) .. " expected:: " .. vim.fn.json_encode(vim.fn.json_decode(vim.g.JiebaTestGroundtruthMark_lsquare)) .. "\\n")
EOF
    else
        execute "!echo " . shellescape("unexpected state_after in mark '[" . " actual:: " . escape(json_encode(getpos("'[")), "\\\\") . " expected:: " . escape(json_encode(json_decode(g:JiebaTestGroundtruthMark_lsquare)), "\\\\"), 1) . " >&2"
    endif
    cquit
    finish
endif
if getpos("']") !=# json_decode(g:JiebaTestGroundtruthMark_rsquare)
    if has("nvim")
        lua <<EOF
io.stderr:write("unexpected state_after in mark ']" .. " actual:: " .. vim.fn.json_encode(vim.fn.getpos("']")) .. " expected:: " .. vim.fn.json_encode(vim.fn.json_decode(vim.g.JiebaTestGroundtruthMark_rsquare)) .. "\\n")
EOF
    else
        execute "!echo " . shellescape("unexpected state_after in mark ']" . " actual:: " . escape(json_encode(getpos("']")), "\\\\") . " expected:: " . escape(json_encode(json_decode(g:JiebaTestGroundtruthMark_rsquare)), "\\\\"), 1) . " >&2"
    endif
    cquit
    finish
endif

" buffer_after checking
if getcurpos() !=# json_decode(g:JiebaTestGroundtruthCursor)
    if has("nvim")
        lua <<EOF
io.stderr:write("unexpected cursor position in buffer_after" .. " actual:: " .. vim.fn.json_encode(vim.fn.getcurpos()) .. " expected:: " .. vim.fn.json_encode(vim.fn.json_decode(vim.g.JiebaTestGroundtruthCursor)) .. "\\n")
EOF
    else
        execute "!echo " . shellescape("unexpected cursor position in buffer_after" . " actual:: " . escape(json_encode(getcurpos()), "\\\\") . " expected:: " . escape(json_encode(json_decode(g:JiebaTestGroundtruthCursor)), "\\\\"), 1) . " >&2"
    endif
    cquit
    finish
endif
normal! gvomaomb
if getpos("'a") !=# json_decode(g:JiebaTestGroundtruthVisualBegin)
    if has("nvim")
        lua <<EOF
io.stderr:write("unexpected visual_begin position in buffer_after" .. " actual:: " .. vim.fn.json_encode(vim.fn.getpos("'a")) .. " expected:: " .. vim.fn.json_encode(vim.fn.json_decode(vim.g.JiebaTestGroundtruthVisualBegin)) .. "\\n")
EOF
    else
        execute "!echo " . shellescape("unexpected visual_begin position in buffer_after" . " actual:: " . escape(json_encode(getpos("'a")), "\\\\") . " expected:: " . escape(json_encode(json_decode(g:JiebaTestGroundtruthVisualBegin)), "\\\\"), 1) . " >&2"
    endif
    cquit
    finish
endif
if getpos("'b") !=# json_decode(g:JiebaTestGroundtruthVisualEnd)
    if has("nvim")
        lua <<EOF
io.stderr:write("unexpected visual_end position in buffer_after" .. " actual:: " .. vim.fn.json_encode(vim.fn.getpos("'b")) .. " expected:: " .. vim.fn.json_encode(vim.fn.json_decode(vim.g.JiebaTestGroundtruthVisualEnd)) .. "\\n")
EOF
    else
        execute "!echo " . shellescape("unexpected visual_end position in buffer_after" . " actual:: " . escape(json_encode(getpos("'b")), "\\\\") . " expected:: " . escape(json_encode(json_decode(g:JiebaTestGroundtruthVisualEnd)), "\\\\"), 1) . " >&2"
    endif
    cquit
    finish
endif

" model_output echoing
if has("nvim")
    lua <<EOF
io.write(vim.fn.json_encode({i = vim.g.model_input, o = vim.g.model_output}) .. "\\n")
EOF
else
    execute "!echo " . shellescape(escape(json_encode({"i": g:model_input, "o": g:model_output}), "\\\\"), 1)
endif

silent xit
"""
    )

    sbuf = StringIO()
    basic_integrated_blocks[2].write_std_run(sbuf)
    assert (
        sbuf.getvalue()
        == """\
if has("nvim")
    if has("nvim")
        lua <<EOF
io.write(vim.fn.json_encode({cf = "continue"}) .. "\\n")
EOF
    else
        execute "!echo " . shellescape(escape(json_encode({"cf": "continue"}), "\\\\"), 1)
    endif
endif

" define oracle model
function! JiebaOracleModel(...)
    let g:model_input = a:000
    let g:model_output = call(function("JiebaModelOmap"), a:000)
    return g:model_output
endfunction

" state_before setup

" buffer_before setup
call setpos(".", [0, 1, 2, 0, 2])

" autocmd setup
function! IncrementAutocmdEventCount(event_name)
    let l:count = get(g:jieba_test_case_events_count, a:event_name, 0)
    let g:jieba_test_case_events_count[a:event_name] = l:count + 1
endfunction

augroup jieba_test_case_autocmd_events_monitoring
    autocmd!
augroup END

" state_before checking


let g:jieba_test_case_events_count = {}
" cursor movement
normal! "ad2W
execute "normal! \\<Esc>"

let s:jieba_test_case_events_count_frozen = copy(g:jieba_test_case_events_count)

" autocmd event counts querying
let g:JiebaTestGroundtruthAutocmdEventsCount = json_encode(s:jieba_test_case_events_count_frozen)

" state_after querying
let g:JiebaTestGroundtruthReg_a = getreg('a')
let g:JiebaTestGroundtruthMark_lsquare = json_encode(getpos("'["))
let g:JiebaTestGroundtruthMark_rsquare = json_encode(getpos("']"))
let g:JiebaTestGroundtruthMark_langle = json_encode(getpos("'<"))
let g:JiebaTestGroundtruthMark_rangle = json_encode(getpos("'>"))

" buffer_after querying
let g:JiebaTestGroundtruthCursor = json_encode(getcurpos())

execute "mksession! " . expand("%:p:h") . "/Session.vim"
silent xit
"""
    )
    sbuf = StringIO()
    basic_integrated_blocks[2].write_custom_run(sbuf)
    assert (
        sbuf.getvalue()
        == """\
if has("nvim")
    if has("nvim")
        lua <<EOF
io.write(vim.fn.json_encode({cf = "continue"}) .. "\\n")
EOF
    else
        execute "!echo " . shellescape(escape(json_encode({"cf": "continue"}), "\\\\"), 1)
    endif
endif

silent execute "source " . expand("%:p:h") . "/Session.vim"

" define oracle model
function! JiebaOracleModel(...)
    let g:model_input = a:000
    let g:model_output = call(function("JiebaModelOmap"), a:000)
    return g:model_output
endfunction

" state_before setup

" buffer_before setup
call setpos(".", [0, 1, 2, 0, 2])

" autocmd setup
function! IncrementAutocmdEventCount(event_name)
    let l:count = get(g:jieba_test_case_events_count, a:event_name, 0)
    let g:jieba_test_case_events_count[a:event_name] = l:count + 1
endfunction

augroup jieba_test_case_autocmd_events_monitoring
    autocmd!
augroup END

" state_before checking


let g:jieba_test_case_events_count = {}
" cursor movement
call JiebaOmap("W", 0, 2, "d", "a", "JiebaOracleModel")
execute "normal! \\<Esc>"

let g:jieba_test_case_events_count_frozen = copy(g:jieba_test_case_events_count)

" autocmd event counts checking
if g:jieba_test_case_events_count_frozen !=# json_decode(g:JiebaTestGroundtruthAutocmdEventsCount)
    if has("nvim")
        lua <<EOF
io.stderr:write("unexpected autocmd events count" .. " actual:: " .. vim.fn.json_encode(vim.g.jieba_test_case_events_count_frozen) .. " expected:: " .. vim.fn.json_encode(vim.fn.json_decode(vim.g.JiebaTestGroundtruthAutocmdEventsCount)) .. "\\n")
EOF
    else
        execute "!echo " . shellescape("unexpected autocmd events count" . " actual:: " . escape(json_encode(g:jieba_test_case_events_count_frozen), "\\\\") . " expected:: " . escape(json_encode(json_decode(g:JiebaTestGroundtruthAutocmdEventsCount)), "\\\\"), 1) . " >&2"
    endif
    cquit
    finish
endif

" state_after checking
if getreg("a") !=# g:JiebaTestGroundtruthReg_a
    if has("nvim")
        lua <<EOF
io.stderr:write("unexpected state_after in register \\"a" .. " actual:: " .. vim.fn.json_encode(vim.fn.getreg("a")) .. " expected:: " .. vim.fn.json_encode(vim.g.JiebaTestGroundtruthReg_a) .. "\\n")
EOF
    else
        execute "!echo " . shellescape("unexpected state_after in register \\"a" . " actual:: " . escape(json_encode(getreg("a")), "\\\\") . " expected:: " . escape(json_encode(g:JiebaTestGroundtruthReg_a), "\\\\"), 1) . " >&2"
    endif
    cquit
    finish
endif
if getpos("'[") !=# json_decode(g:JiebaTestGroundtruthMark_lsquare)
    if has("nvim")
        lua <<EOF
io.stderr:write("unexpected state_after in mark '[" .. " actual:: " .. vim.fn.json_encode(vim.fn.getpos("'[")) .. " expected:: " .. vim.fn.json_encode(vim.fn.json_decode(vim.g.JiebaTestGroundtruthMark_lsquare)) .. "\\n")
EOF
    else
        execute "!echo " . shellescape("unexpected state_after in mark '[" . " actual:: " . escape(json_encode(getpos("'[")), "\\\\") . " expected:: " . escape(json_encode(json_decode(g:JiebaTestGroundtruthMark_lsquare)), "\\\\"), 1) . " >&2"
    endif
    cquit
    finish
endif
if getpos("']") !=# json_decode(g:JiebaTestGroundtruthMark_rsquare)
    if has("nvim")
        lua <<EOF
io.stderr:write("unexpected state_after in mark ']" .. " actual:: " .. vim.fn.json_encode(vim.fn.getpos("']")) .. " expected:: " .. vim.fn.json_encode(vim.fn.json_decode(vim.g.JiebaTestGroundtruthMark_rsquare)) .. "\\n")
EOF
    else
        execute "!echo " . shellescape("unexpected state_after in mark ']" . " actual:: " . escape(json_encode(getpos("']")), "\\\\") . " expected:: " . escape(json_encode(json_decode(g:JiebaTestGroundtruthMark_rsquare)), "\\\\"), 1) . " >&2"
    endif
    cquit
    finish
endif
if getpos("'<") !=# json_decode(g:JiebaTestGroundtruthMark_langle)
    if has("nvim")
        lua <<EOF
io.stderr:write("unexpected state_after in mark '<" .. " actual:: " .. vim.fn.json_encode(vim.fn.getpos("'<")) .. " expected:: " .. vim.fn.json_encode(vim.fn.json_decode(vim.g.JiebaTestGroundtruthMark_langle)) .. "\\n")
EOF
    else
        execute "!echo " . shellescape("unexpected state_after in mark '<" . " actual:: " . escape(json_encode(getpos("'<")), "\\\\") . " expected:: " . escape(json_encode(json_decode(g:JiebaTestGroundtruthMark_langle)), "\\\\"), 1) . " >&2"
    endif
    cquit
    finish
endif
if getpos("'>") !=# json_decode(g:JiebaTestGroundtruthMark_rangle)
    if has("nvim")
        lua <<EOF
io.stderr:write("unexpected state_after in mark '>" .. " actual:: " .. vim.fn.json_encode(vim.fn.getpos("'>")) .. " expected:: " .. vim.fn.json_encode(vim.fn.json_decode(vim.g.JiebaTestGroundtruthMark_rangle)) .. "\\n")
EOF
    else
        execute "!echo " . shellescape("unexpected state_after in mark '>" . " actual:: " . escape(json_encode(getpos("'>")), "\\\\") . " expected:: " . escape(json_encode(json_decode(g:JiebaTestGroundtruthMark_rangle)), "\\\\"), 1) . " >&2"
    endif
    cquit
    finish
endif

" buffer_after checking
if getcurpos() !=# json_decode(g:JiebaTestGroundtruthCursor)
    if has("nvim")
        lua <<EOF
io.stderr:write("unexpected cursor position in buffer_after" .. " actual:: " .. vim.fn.json_encode(vim.fn.getcurpos()) .. " expected:: " .. vim.fn.json_encode(vim.fn.json_decode(vim.g.JiebaTestGroundtruthCursor)) .. "\\n")
EOF
    else
        execute "!echo " . shellescape("unexpected cursor position in buffer_after" . " actual:: " . escape(json_encode(getcurpos()), "\\\\") . " expected:: " . escape(json_encode(json_decode(g:JiebaTestGroundtruthCursor)), "\\\\"), 1) . " >&2"
    endif
    cquit
    finish
endif

" model_output echoing
if has("nvim")
    lua <<EOF
io.write(vim.fn.json_encode({i = vim.g.model_input, o = vim.g.model_output}) .. "\\n")
EOF
else
    execute "!echo " . shellescape(escape(json_encode({"i": g:model_input, "o": g:model_output}), "\\\\"), 1)
endif

silent xit
"""
    )
