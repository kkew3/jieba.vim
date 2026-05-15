import pytest

from . import parser as m


def test_raw_block_new():
    span = m.SourceSpan()
    b = m.RawBlock(
        [
            m.RawDirective("M", "n", span.copy_as(3)),
            m.RawDirective("X", "u", span.copy_as(4)),
            m.RawDirective("K", "w", span.copy_as(6)),
            m.RawDirective("X", "b", span.copy_as(7)),
            m.RawDirective("C", "1", span.copy_as(5)),
        ]
    )
    assert b.directives == [
        m.RawDirective("C", "1", span.copy_as(5)),
        m.RawDirective("K", "w", span.copy_as(6)),
        m.RawDirective("M", "n", span.copy_as(3)),
        m.RawDirective("X", "u", span.copy_as(4)),
        m.RawDirective("X", "b", span.copy_as(7)),
    ]
    assert b.span == span.copy_as(3, 7)


def test_raw_block_extend_defaults():
    span = m.SourceSpan()
    b = m.RawBlock(
        [
            m.RawDirective("M", "n", span.copy_as(13)),
            m.RawDirective("X", "u", span.copy_as(14)),
            m.RawDirective("K", "w", span.copy_as(16)),
            m.RawDirective("X", "b", span.copy_as(17)),
            m.RawDirective("C", "1", span.copy_as(15)),
        ]
    )
    defaults = [
        m.RawDirective("M", "n", span.copy_as(1)),
        m.RawDirective("C", "0", span.copy_as(2)),
        m.RawDirective("S0", '"a=foo', span.copy_as(3)),
        m.RawDirective("E", "CmdlineEnter=", span.copy_as(4)),
        m.RawDirective("E", "CursorMoved=", span.copy_as(5)),
        m.RawDirective("X", "bi", span.copy_as(6)),
    ]
    b.extend_defaults(defaults)
    assert b.directives == [
        m.RawDirective("C", "1", span.copy_as(15)),
        m.RawDirective("E", "CmdlineEnter=", span.copy_as(4)),
        m.RawDirective("E", "CursorMoved=", span.copy_as(5)),
        m.RawDirective("K", "w", span.copy_as(16)),
        m.RawDirective("M", "n", span.copy_as(13)),
        m.RawDirective("S0", '"a=foo', span.copy_as(3)),
        m.RawDirective("X", "u", span.copy_as(14)),
        m.RawDirective("X", "b", span.copy_as(17)),
    ]
    assert b.span == span.copy_as(13, 17)


def test_raw_test_cases_extend_from_lines():
    raw_test_cases = m.RawTestCases()
    span = m.SourceSpan.for_file("foo")
    lines = """
#V 4
##

? !has:nvim

#M n
#X bi
#S0 "a=foo

K w
C 3
B0 |foo␊
E CursorMoved= CursorMovedI=
E CmdlineEnter=
S1 "a=foo

K b
X u
B0 fo|o␊
S0 "a=bar
E CmdlineEnter=
S1 "a=bar
"""
    raw_test_cases.extend_from_lines(lines.splitlines(), span)
    assert len(raw_test_cases.blocks) == 2
    assert raw_test_cases.blocks[0].directives == [
        m.RawDirective("?", "!has:nvim", span.copy_as(5)),
        m.RawDirective("B0", "|foo␊", span.copy_as(13)),
        m.RawDirective("C", "3", span.copy_as(12)),
        m.RawDirective("E", "CursorMoved=", span.copy_as(14)),
        m.RawDirective("E", "CursorMovedI=", span.copy_as(14)),
        m.RawDirective("E", "CmdlineEnter=", span.copy_as(15)),
        m.RawDirective("K", "w", span.copy_as(11)),
        m.RawDirective("M", "n", span.copy_as(7)),
        m.RawDirective("S0", '"a=foo', span.copy_as(9)),
        m.RawDirective("S1", '"a=foo', span.copy_as(16)),
        m.RawDirective("V", "4", span.copy_as(2)),
        m.RawDirective("X", "bi", span.copy_as(8)),
    ]
    assert raw_test_cases.blocks[0].span == span.copy_as(11, 16)
    assert raw_test_cases.blocks[1].directives == [
        m.RawDirective("?", "!has:nvim", span.copy_as(5)),
        m.RawDirective("B0", "fo|o␊", span.copy_as(20)),
        m.RawDirective("E", "CmdlineEnter=", span.copy_as(22)),
        m.RawDirective("K", "b", span.copy_as(18)),
        m.RawDirective("M", "n", span.copy_as(7)),
        m.RawDirective("S0", '"a=bar', span.copy_as(21)),
        m.RawDirective("S1", '"a=bar', span.copy_as(23)),
        m.RawDirective("V", "4", span.copy_as(2)),
        m.RawDirective("X", "u", span.copy_as(19)),
    ]
    assert raw_test_cases.blocks[1].span == span.copy_as(18, 23)


def test_parse_state_expr():
    p = m.StateExpr.parse

    span = m.SourceSpan()
    assert p("visualmode()=v", span) == m.StateExpr("func", "visualmode", "v")
    assert p("visualmode()=", span, parse_as_incomplete=True) == m.StateExpr(
        "func", "visualmode", None
    )
    assert p('"a=foo', span) == m.StateExpr("reg", "a", "foo")
    assert p('"a=', span) == m.StateExpr("reg", "a", "")
    assert p('"a=', span, parse_as_incomplete=True) == m.StateExpr(
        "reg", "a", None
    )
    assert p("'a=[0,1,2,0]", span) == m.StateExpr("mark", "a", [0, 1, 2, 0])
    assert p("'a=", span, parse_as_incomplete=True) == m.StateExpr(
        "mark", "a", None
    )
    assert p("selection=inclusive", span) == m.StateExpr(
        "opt", "selection", "inclusive"
    )


def test_parse_buffer_expr():
    span = m.SourceSpan.for_file("foo").copy_as(3)
    assert m.BufferExpr.parse("␀", span) == m.BufferExpr(
        clean_buffer=[],
        langle=None,
        rangle=None,
        visual_begin=None,
        visual_end=None,
        cursor=None,
    )
    assert m.BufferExpr.parse("<|>␀", span) == m.BufferExpr(
        clean_buffer=[],
        langle=[0, 1, 1, 0],
        rangle=[0, 1, 1, 0],
        visual_begin=None,
        visual_end=None,
        cursor=[0, 1, 1, 0, 1],
    )
    assert m.BufferExpr.parse("<|>\\␀", span) == m.BufferExpr(
        clean_buffer=[],
        langle=[0, 1, 1, 0],
        rangle=[0, 1, 1, 0],
        visual_begin=None,
        visual_end=None,
        cursor=[0, 1, 1, 0, 2147483647],
    )
    assert m.BufferExpr.parse("<|>\\␀~", span) == m.BufferExpr(
        clean_buffer=[],
        langle=[0, 1, 1, 0],
        rangle=[0, 1, 1, 0],
        visual_begin=None,
        visual_end=None,
        cursor=[0, 1, 1, 0, 1],
    )
    assert m.BufferExpr.parse("<|>␀\\~", span) == m.BufferExpr(
        clean_buffer=[],
        langle=[0, 1, 1, 0],
        rangle=[0, 1, 1, 0],
        visual_begin=None,
        visual_end=None,
        cursor=[0, 1, 1, 0, 2],
    )
    assert m.BufferExpr.parse("<>␀|\\~", span) == m.BufferExpr(
        clean_buffer=[],
        langle=[0, 1, 1, 0],
        rangle=[0, 1, 1, 0],
        visual_begin=None,
        visual_end=None,
        cursor=[0, 1, 1, 1, 2],
    )
    assert m.BufferExpr.parse("␊", span) == m.BufferExpr(
        clean_buffer=[""],
        langle=None,
        rangle=None,
        visual_begin=None,
        visual_end=None,
        cursor=None,
    )
    assert m.BufferExpr.parse("|␊", span) == m.BufferExpr(
        clean_buffer=[""],
        langle=None,
        rangle=None,
        visual_begin=None,
        visual_end=None,
        cursor=[0, 1, 1, 0, 1],
    )
    assert m.BufferExpr.parse("|\\␊", span) == m.BufferExpr(
        clean_buffer=[""],
        langle=None,
        rangle=None,
        visual_begin=None,
        visual_end=None,
        cursor=[0, 1, 1, 0, 2147483647],
    )
    assert m.BufferExpr.parse("|\\␊~", span) == m.BufferExpr(
        clean_buffer=[""],
        langle=None,
        rangle=None,
        visual_begin=None,
        visual_end=None,
        cursor=[0, 1, 1, 0, 1],
    )
    assert m.BufferExpr.parse("abc·|def␊", span) == m.BufferExpr(
        clean_buffer=["abc def"],
        langle=None,
        rangle=None,
        visual_begin=None,
        visual_end=None,
        cursor=[0, 1, 5, 0, 5],
    )
    assert m.BufferExpr.parse("<[ab]c·|def\\>␊", span) == m.BufferExpr(
        clean_buffer=["abc def"],
        langle=[0, 1, 1, 0],
        rangle=[0, 1, 8, 0],
        visual_begin=[0, 1, 1, 0],
        visual_end=[0, 1, 3, 0],
        cursor=[0, 1, 5, 0, 2147483647],
    )
    assert m.BufferExpr.parse("<[ab]c·|de\\f>␊", span) == m.BufferExpr(
        clean_buffer=["abc def"],
        langle=[0, 1, 1, 0],
        rangle=[0, 1, 8, 0],
        visual_begin=[0, 1, 1, 0],
        visual_end=[0, 1, 3, 0],
        cursor=[0, 1, 5, 0, 7],
    )
    assert m.BufferExpr.parse("aa␊|e␊cc␊", span) == m.BufferExpr(
        clean_buffer=["aa", "e", "cc"],
        langle=None,
        rangle=None,
        visual_begin=None,
        visual_end=None,
        cursor=[0, 2, 1, 0, 1],
    )
    assert m.BufferExpr.parse("abc┤~~|~~def␊~~\\~~~gh␊", span) == m.BufferExpr(
        clean_buffer=["abc\tdef", "gh"],
        langle=None,
        rangle=None,
        visual_begin=None,
        visual_end=None,
        cursor=[0, 1, 4, 3, 15],
    )
    assert m.BufferExpr.parse("abc┤~~|~~def\\␊~~~~~gh␊", span) == m.BufferExpr(
        clean_buffer=["abc\tdef", "gh"],
        langle=None,
        rangle=None,
        visual_begin=None,
        visual_end=None,
        cursor=[0, 1, 4, 3, 12],
    )
    assert m.BufferExpr.parse("abc@@|d␊efgh␊", span) == m.BufferExpr(
        clean_buffer=["abcd", "efgh"],
        langle=None,
        rangle=None,
        visual_begin=None,
        visual_end=None,
        cursor=[0, 1, 6, 0, 6],
    )
    for invalid_expr in [
        "␊␀",
        "␀␊",
        "abc␀",
        "␀abc␊",
        "~abc·def␊",
        "|abc␊\\def␊",
        "|abc·def",
        "\\abc·def␊",
        "|abc·|def␊",
        "abc·\\d\\ef␊",
        "<<abc·def␊",
    ]:
        with pytest.raises(m.ParseError):
            m.BufferExpr.parse(invalid_expr, span)


def test_parse_autocmd_event_count_expr():
    span = m.SourceSpan.for_file("foo").copy_as(3)
    assert m.AutocmdEventCountExpr.parse(
        "CmdlineEnter=1", span
    ) == m.AutocmdEventCountExpr("CmdlineEnter", 1)
    assert m.AutocmdEventCountExpr.parse(
        "CmdlineEnter=", span, parse_as_incomplete=True
    ) == m.AutocmdEventCountExpr("CmdlineEnter", None)
    with pytest.raises(m.ParseError):
        m.AutocmdEventCountExpr.parse("CmdlineEnter=", span)
