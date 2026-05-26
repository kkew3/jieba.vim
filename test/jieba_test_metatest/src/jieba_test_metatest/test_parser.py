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
