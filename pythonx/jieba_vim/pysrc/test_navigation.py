import pytest

from .navigation import *


def test_parse_tokens():
    tokens = ['Pixelmator', ' ', 'Pro', '在', '设计', '，', '完全']
    expected = [(0, 9, TokenType.hans), (10, 10, TokenType.space),
                (11, 13, TokenType.hans), (14, 14, TokenType.hans),
                (17, 20, TokenType.hans), (23, 23, TokenType.punc),
                (26, 29, TokenType.hans)]
    expected = list(ParsedToken(*x) for x in expected)
    assert parse_tokens(tokens) == expected


def test_index_tokens():
    with pytest.raises(IndexError):
        index_tokens([], 0)

    pt = [(0, 9, TokenType.hans), (10, 10, TokenType.space),
          (11, 13, TokenType.hans), (14, 14, TokenType.hans),
          (17, 20, TokenType.hans)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_tokens(pt, 0) == 0
    assert index_tokens(pt, 9) == 0
    assert index_tokens(pt, 10) == 1
    assert index_tokens(pt, 11) == 2
    assert index_tokens(pt, 13) == 2
    assert index_tokens(pt, 14) == 3
    assert index_tokens(pt, 16) == 3
    assert index_tokens(pt, 17) == 4
    assert index_tokens(pt, 20) == 4
    assert index_tokens(pt, 21) == 4


def test_index_last_start_of_PorH():
    pt = []
    assert index_last_start_of_PorH(pt) == 0
    pt = [(0, 2, TokenType.space)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_last_start_of_PorH(pt) is None
    pt = [(0, 2, TokenType.space), (3, 4, TokenType.hans)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_last_start_of_PorH(pt) == 3
    pt = [(0, 1, TokenType.hans), (2, 3, TokenType.hans)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_last_start_of_PorH(pt) == 2
    pt = [(0, 1, TokenType.hans), (2, 3, TokenType.hans),
          (4, 4, TokenType.punc)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_last_start_of_PorH(pt) == 4
    pt = [(0, 3, TokenType.hans), (4, 4, TokenType.hans),
          (7, 10, TokenType.punc), (13, 17, TokenType.space)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_last_start_of_PorH(pt) == 7
    pt = [(0, 0, TokenType.punc), (3, 3, TokenType.punc)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_last_start_of_PorH(pt) == 3


def test_index_prev_start_of_PorH():
    pt = []
    assert index_prev_start_of_PorH(pt, 0) is None
    pt = [(0, 3, TokenType.hans)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_prev_start_of_PorH(pt, 0) is None
    assert index_prev_start_of_PorH(pt, 1) == 0
    assert index_prev_start_of_PorH(pt, 3) == 0
    pt = [(0, 3, TokenType.hans), (4, 5, TokenType.hans)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_prev_start_of_PorH(pt, 0) is None
    assert index_prev_start_of_PorH(pt, 1) == 0
    assert index_prev_start_of_PorH(pt, 3) == 0
    assert index_prev_start_of_PorH(pt, 4) == 0
    assert index_prev_start_of_PorH(pt, 5) == 4
    pt = [(0, 3, TokenType.hans), (6, 9, TokenType.hans)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_prev_start_of_PorH(pt, 0) is None
    assert index_prev_start_of_PorH(pt, 1) == 0
    assert index_prev_start_of_PorH(pt, 3) == 0
    assert index_prev_start_of_PorH(pt, 4) == 0
    assert index_prev_start_of_PorH(pt, 6) == 0
    assert index_prev_start_of_PorH(pt, 7) == 6
    assert index_prev_start_of_PorH(pt, 9) == 6
    pt = [(0, 3, TokenType.hans), (4, 4, TokenType.punc),
          (5, 6, TokenType.space)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_prev_start_of_PorH(pt, 5) == 4
    assert index_prev_start_of_PorH(pt, 6) == 4
    pt = [(0, 1, TokenType.space), (2, 2, TokenType.hans),
          (5, 5, TokenType.punc), (8, 9, TokenType.space)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_prev_start_of_PorH(pt, 0) is None
    assert index_prev_start_of_PorH(pt, 1) is None
    assert index_prev_start_of_PorH(pt, 2) is None
    assert index_prev_start_of_PorH(pt, 3) == 2
    assert index_prev_start_of_PorH(pt, 4) == 2
    assert index_prev_start_of_PorH(pt, 5) == 2
    assert index_prev_start_of_PorH(pt, 6) == 5
    assert index_prev_start_of_PorH(pt, 8) == 5
    assert index_prev_start_of_PorH(pt, 9) == 5
    pt = [(0, 0, TokenType.punc), (3, 3, TokenType.punc)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_prev_start_of_PorH(pt, 3) == 0
    assert index_prev_start_of_PorH(pt, 4) == 3


def test_index_last_start_of_nonS():
    pt = []
    assert index_last_start_of_nonS(pt) == 0
    pt = [(0, 1, TokenType.hans), (2, 3, TokenType.hans)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_last_start_of_nonS(pt) == 2
    pt = [(0, 3, TokenType.hans), (4, 4, TokenType.hans),
          (7, 10, TokenType.punc), (13, 17, TokenType.space)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_last_start_of_nonS(pt) == 4
    pt = [(0, 3, TokenType.hans), (4, 4, TokenType.punc),
          (7, 10, TokenType.hans), (13, 17, TokenType.space)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_last_start_of_nonS(pt) == 7
    pt = [(0, 1, TokenType.space), (2, 5, TokenType.hans)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_last_start_of_nonS(pt) == 2
    pt = [(0, 1, TokenType.space), (2, 5, TokenType.hans),
          (8, 8, TokenType.space)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_last_start_of_nonS(pt) == 2
    pt = [(0, 0, TokenType.punc), (3, 3, TokenType.punc)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_last_start_of_nonS(pt) == 3


def test_index_prev_start_of_nonS():
    pt = []
    assert index_prev_start_of_nonS(pt, 0) is None
    pt = [(0, 1, TokenType.space), (2, 2, TokenType.hans),
          (5, 5, TokenType.punc), (8, 9, TokenType.space),
          (10, 10, TokenType.punc), (13, 13, TokenType.hans),
          (16, 16, TokenType.hans)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_prev_start_of_nonS(pt, 0) is None
    assert index_prev_start_of_nonS(pt, 1) is None
    assert index_prev_start_of_nonS(pt, 2) is None
    assert index_prev_start_of_nonS(pt, 3) == 2
    assert index_prev_start_of_nonS(pt, 5) == 2
    assert index_prev_start_of_nonS(pt, 6) == 2
    assert index_prev_start_of_nonS(pt, 7) == 2
    assert index_prev_start_of_nonS(pt, 9) == 2
    assert index_prev_start_of_nonS(pt, 10) == 2
    assert index_prev_start_of_nonS(pt, 11) == 10
    assert index_prev_start_of_nonS(pt, 13) == 10
    assert index_prev_start_of_nonS(pt, 14) == 13
    assert index_prev_start_of_nonS(pt, 15) == 13
    assert index_prev_start_of_nonS(pt, 16) == 13
    assert index_prev_start_of_nonS(pt, 17) == 16
    pt = [(0, 0, TokenType.punc), (3, 3, TokenType.punc)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_prev_start_of_nonS(pt, 3) == 0
    assert index_prev_start_of_nonS(pt, 4) == 3


def test_index_last_end_of_PorH():
    pt = []
    assert index_last_end_of_PorH(pt) == 0
    pt = [(0, 2, TokenType.space)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_last_end_of_PorH(pt) is None
    pt = [(0, 2, TokenType.space), (3, 4, TokenType.hans)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_last_end_of_PorH(pt) == 4
    pt = [(0, 1, TokenType.hans), (2, 3, TokenType.hans)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_last_end_of_PorH(pt) == 3
    pt = [(0, 1, TokenType.hans), (2, 3, TokenType.hans),
          (4, 4, TokenType.punc)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_last_end_of_PorH(pt) == 4
    pt = [(0, 3, TokenType.hans), (4, 4, TokenType.hans),
          (7, 10, TokenType.punc), (13, 17, TokenType.space)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_last_end_of_PorH(pt) == 10
    pt = [(0, 0, TokenType.punc), (3, 3, TokenType.punc)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_last_end_of_PorH(pt) == 3


def test_index_prev_end_of_PorH():
    pt = []
    assert index_prev_end_of_PorH(pt, 0) is None
    pt = [(0, 3, TokenType.hans)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_prev_end_of_PorH(pt, 0) is None
    assert index_prev_end_of_PorH(pt, 1) is None
    assert index_prev_end_of_PorH(pt, 3) is None
    pt = [(0, 3, TokenType.hans), (4, 5, TokenType.hans)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_prev_end_of_PorH(pt, 0) is None
    assert index_prev_end_of_PorH(pt, 1) is None
    assert index_prev_end_of_PorH(pt, 3) is None
    assert index_prev_end_of_PorH(pt, 4) == 3
    assert index_prev_end_of_PorH(pt, 5) == 3
    pt = [(0, 3, TokenType.hans), (6, 9, TokenType.hans)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_prev_end_of_PorH(pt, 0) is None
    assert index_prev_end_of_PorH(pt, 1) is None
    assert index_prev_end_of_PorH(pt, 3) is None
    assert index_prev_end_of_PorH(pt, 4) is None
    assert index_prev_end_of_PorH(pt, 5) is None
    assert index_prev_end_of_PorH(pt, 6) == 3
    assert index_prev_end_of_PorH(pt, 7) == 3
    assert index_prev_end_of_PorH(pt, 9) == 3
    pt = [(0, 3, TokenType.hans), (4, 4, TokenType.punc),
          (5, 6, TokenType.space)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_prev_end_of_PorH(pt, 5) == 4
    assert index_prev_end_of_PorH(pt, 6) == 4
    pt = [(0, 1, TokenType.space), (2, 2, TokenType.hans),
          (5, 6, TokenType.punc), (8, 9, TokenType.space)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_prev_end_of_PorH(pt, 0) is None
    assert index_prev_end_of_PorH(pt, 1) is None
    assert index_prev_end_of_PorH(pt, 2) is None
    assert index_prev_end_of_PorH(pt, 3) is None
    assert index_prev_end_of_PorH(pt, 4) is None
    assert index_prev_end_of_PorH(pt, 5) == 2
    assert index_prev_end_of_PorH(pt, 6) == 2
    assert index_prev_end_of_PorH(pt, 7) == 2
    assert index_prev_end_of_PorH(pt, 8) == 6
    assert index_prev_end_of_PorH(pt, 9) == 6
    pt = [(0, 0, TokenType.punc), (3, 3, TokenType.punc)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_prev_end_of_PorH(pt, 4) == 0


def test_index_last_end_of_nonS():
    pt = []
    assert index_last_end_of_nonS(pt) == 0
    pt = [(0, 1, TokenType.hans), (2, 3, TokenType.hans)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_last_end_of_nonS(pt) == 3
    pt = [(0, 3, TokenType.hans), (4, 4, TokenType.hans),
          (7, 10, TokenType.punc), (13, 17, TokenType.space)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_last_end_of_nonS(pt) == 10
    pt = [(0, 3, TokenType.hans), (4, 4, TokenType.punc),
          (7, 10, TokenType.hans), (13, 17, TokenType.space)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_last_end_of_nonS(pt) == 10
    pt = [(0, 1, TokenType.space), (2, 5, TokenType.hans)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_last_end_of_nonS(pt) == 5
    pt = [(0, 1, TokenType.space), (2, 5, TokenType.hans),
          (8, 8, TokenType.space)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_last_end_of_nonS(pt) == 5
    pt = [(0, 0, TokenType.punc), (3, 3, TokenType.punc)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_last_end_of_nonS(pt) == 3


def test_index_prev_end_of_nonS():
    pt = []
    assert index_prev_end_of_nonS(pt, 0) is None
    pt = [(0, 1, TokenType.space), (2, 3, TokenType.hans),
          (5, 5, TokenType.punc), (8, 9, TokenType.space),
          (10, 10, TokenType.punc), (13, 13, TokenType.hans),
          (16, 16, TokenType.hans)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_prev_end_of_nonS(pt, 0) is None
    assert index_prev_end_of_nonS(pt, 1) is None
    assert index_prev_end_of_nonS(pt, 2) is None
    assert index_prev_end_of_nonS(pt, 3) is None
    assert index_prev_end_of_nonS(pt, 5) is None
    assert index_prev_end_of_nonS(pt, 6) is None
    assert index_prev_end_of_nonS(pt, 7) is None
    assert index_prev_end_of_nonS(pt, 8) == 5
    assert index_prev_end_of_nonS(pt, 9) == 5
    assert index_prev_end_of_nonS(pt, 10) == 5
    assert index_prev_end_of_nonS(pt, 11) == 5
    assert index_prev_end_of_nonS(pt, 13) == 10
    assert index_prev_end_of_nonS(pt, 14) == 10
    assert index_prev_end_of_nonS(pt, 15) == 10
    assert index_prev_end_of_nonS(pt, 16) == 13
    assert index_prev_end_of_nonS(pt, 17) == 13
    pt = [(0, 0, TokenType.punc), (3, 3, TokenType.punc)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_prev_end_of_nonS(pt, 4) == 0
