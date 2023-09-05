import pytest

from .navigation import *


def test_parse_tokens():
    tokens = ['Pixelmator', '-', 'Pro', ' ', '在', '设计', '，', '完全']
    expected = [(0, 9, TokenType.hans), (10, 10, TokenType.non_word),
                (11, 13, TokenType.hans), (14, 14, TokenType.space),
                (15, 15, TokenType.hans), (18, 21, TokenType.hans),
                (24, 24, TokenType.punc), (27, 30, TokenType.hans)]
    expected = list(ParsedToken(*x) for x in expected)
    assert parse_tokens(tokens) == expected


def test_stack_merge():
    def _rule(a, b):
        if b % 2 == 1:
            return [a, 999, b + 10]
        return None

    assert stack_merge([0, 1, 2, 3], _rule) == [0, 999, 11, 2, 999, 13]
    assert stack_merge([1, 2, 3], _rule) == [999, 11, 2, 999, 13]


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
    pt = [(0, 9, TokenType.hans), (10, 10, TokenType.space),
          (11, 13, TokenType.hans), (14, 13, TokenType.space),
          (14, 14, TokenType.hans), (17, 16, TokenType.space),
          (17, 20, TokenType.hans)]
    pt = list(ParsedToken(*x) for x in pt)
    assert index_tokens(pt, 14) == 4
    assert index_tokens(pt, 16) == 4
    assert index_tokens(pt, 17) == 6
    assert index_tokens(pt, 20) == 6
    assert index_tokens(pt, 21) == 6


def _form_parsed_tokens(elements):
    return stack_merge((ParsedToken(*x) for x in elements),
                       insert_implicit_space_rule)


def test_index_last_start_of_word():
    pt = []
    assert index_last_start_of_word(pt) == 0
    pt = [(0, 2, TokenType.space)]
    pt = _form_parsed_tokens(pt)
    assert index_last_start_of_word(pt) is None
    pt = [(0, 2, TokenType.space), (3, 4, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_last_start_of_word(pt) == 3
    pt = [(0, 1, TokenType.hans), (2, 3, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_last_start_of_word(pt) == 2
    pt = [(0, 1, TokenType.hans), (2, 3, TokenType.hans),
          (4, 4, TokenType.punc)]
    pt = _form_parsed_tokens(pt)
    assert index_last_start_of_word(pt) == 4
    pt = [(0, 3, TokenType.hans), (4, 4, TokenType.hans),
          (7, 10, TokenType.punc), (13, 17, TokenType.space)]
    pt = _form_parsed_tokens(pt)
    assert index_last_start_of_word(pt) == 7
    pt = [(0, 0, TokenType.punc), (3, 3, TokenType.punc)]
    pt = _form_parsed_tokens(pt)
    assert index_last_start_of_word(pt) == 3


def test_index_prev_start_of_word():
    pt = []
    assert index_prev_start_of_word(pt, 0) is None
    pt = [(0, 3, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_prev_start_of_word(pt, 0) is None
    assert index_prev_start_of_word(pt, 1) == 0
    assert index_prev_start_of_word(pt, 3) == 0
    pt = [(0, 3, TokenType.hans), (4, 5, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_prev_start_of_word(pt, 0) is None
    assert index_prev_start_of_word(pt, 1) == 0
    assert index_prev_start_of_word(pt, 3) == 0
    assert index_prev_start_of_word(pt, 4) == 0
    assert index_prev_start_of_word(pt, 5) == 4
    pt = [(0, 3, TokenType.hans), (6, 9, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_prev_start_of_word(pt, 0) is None
    assert index_prev_start_of_word(pt, 1) == 0
    assert index_prev_start_of_word(pt, 3) == 0
    assert index_prev_start_of_word(pt, 4) == 0
    assert index_prev_start_of_word(pt, 6) == 0
    assert index_prev_start_of_word(pt, 7) == 6
    assert index_prev_start_of_word(pt, 9) == 6
    pt = [(0, 3, TokenType.hans), (4, 4, TokenType.punc),
          (5, 6, TokenType.space)]
    pt = _form_parsed_tokens(pt)
    assert index_prev_start_of_word(pt, 5) == 4
    assert index_prev_start_of_word(pt, 6) == 4
    pt = [(0, 1, TokenType.space), (2, 2, TokenType.hans),
          (5, 5, TokenType.punc), (8, 9, TokenType.space)]
    pt = _form_parsed_tokens(pt)
    assert index_prev_start_of_word(pt, 0) is None
    assert index_prev_start_of_word(pt, 1) is None
    assert index_prev_start_of_word(pt, 2) is None
    assert index_prev_start_of_word(pt, 3) == 2
    assert index_prev_start_of_word(pt, 4) == 2
    assert index_prev_start_of_word(pt, 5) == 2
    assert index_prev_start_of_word(pt, 6) == 5
    assert index_prev_start_of_word(pt, 8) == 5
    assert index_prev_start_of_word(pt, 9) == 5
    pt = [(0, 0, TokenType.punc), (3, 3, TokenType.punc)]
    pt = _form_parsed_tokens(pt)
    assert index_prev_start_of_word(pt, 3) == 0
    assert index_prev_start_of_word(pt, 4) == 3


def test_index_last_start_of_WORD():
    pt = []
    assert index_last_start_of_WORD(pt) == 0
    pt = [(0, 1, TokenType.hans), (2, 3, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_last_start_of_WORD(pt) == 2
    pt = [(0, 3, TokenType.hans), (4, 4, TokenType.hans),
          (7, 10, TokenType.punc), (13, 17, TokenType.space)]
    pt = _form_parsed_tokens(pt)
    assert index_last_start_of_WORD(pt) == 4
    pt = [(0, 3, TokenType.hans), (4, 4, TokenType.punc),
          (7, 10, TokenType.hans), (13, 17, TokenType.space)]
    pt = _form_parsed_tokens(pt)
    assert index_last_start_of_WORD(pt) == 7
    pt = [(0, 1, TokenType.space), (2, 5, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_last_start_of_WORD(pt) == 2
    pt = [(0, 1, TokenType.space), (2, 5, TokenType.hans),
          (8, 8, TokenType.space)]
    pt = _form_parsed_tokens(pt)
    assert index_last_start_of_WORD(pt) == 2
    pt = [(0, 0, TokenType.punc), (3, 3, TokenType.punc)]
    pt = _form_parsed_tokens(pt)
    assert index_last_start_of_WORD(pt) == 3
    pt = [(0, 1, TokenType.hans), (2, 2, TokenType.punc),
          (5, 5, TokenType.non_word), (6, 6, TokenType.punc)]
    pt = _form_parsed_tokens(pt)
    assert index_last_start_of_WORD(pt) == 5
    pt = [(0, 1, TokenType.hans), (2, 2, TokenType.hans),
          (5, 5, TokenType.non_word), (6, 7, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_last_start_of_WORD(pt) == 2
    pt = [(0, 1, TokenType.hans), (2, 2, TokenType.hans),
          (5, 5, TokenType.non_word), (6, 7, TokenType.punc)]
    pt = _form_parsed_tokens(pt)
    assert index_last_start_of_WORD(pt) == 2
    pt = [(0, 1, TokenType.hans), (2, 2, TokenType.punc),
          (5, 5, TokenType.non_word), (6, 7, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_last_start_of_WORD(pt) == 5


def test_index_prev_start_of_WORD():
    pt = []
    assert index_prev_start_of_WORD(pt, 0) is None
    pt = [(0, 1, TokenType.space), (2, 2, TokenType.hans),
          (5, 5, TokenType.punc), (8, 9, TokenType.space),
          (10, 10, TokenType.punc), (13, 13, TokenType.hans),
          (16, 16, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_prev_start_of_WORD(pt, 0) is None
    assert index_prev_start_of_WORD(pt, 1) is None
    assert index_prev_start_of_WORD(pt, 2) is None
    assert index_prev_start_of_WORD(pt, 3) == 2
    assert index_prev_start_of_WORD(pt, 5) == 2
    assert index_prev_start_of_WORD(pt, 6) == 2
    assert index_prev_start_of_WORD(pt, 7) == 2
    assert index_prev_start_of_WORD(pt, 9) == 2
    assert index_prev_start_of_WORD(pt, 10) == 2
    assert index_prev_start_of_WORD(pt, 11) == 10
    assert index_prev_start_of_WORD(pt, 13) == 10
    assert index_prev_start_of_WORD(pt, 14) == 13
    assert index_prev_start_of_WORD(pt, 15) == 13
    assert index_prev_start_of_WORD(pt, 16) == 13
    assert index_prev_start_of_WORD(pt, 17) == 16
    pt = [(0, 0, TokenType.punc), (3, 3, TokenType.punc)]
    pt = _form_parsed_tokens(pt)
    assert index_prev_start_of_WORD(pt, 3) == 0
    assert index_prev_start_of_WORD(pt, 4) == 3


def test_index_last_end_of_word():
    pt = []
    assert index_last_end_of_word(pt) == 0
    pt = [(0, 2, TokenType.space)]
    pt = _form_parsed_tokens(pt)
    assert index_last_end_of_word(pt) is None
    pt = [(0, 2, TokenType.space), (3, 4, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_last_end_of_word(pt) == 4
    pt = [(0, 1, TokenType.hans), (2, 3, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_last_end_of_word(pt) == 3
    pt = [(0, 1, TokenType.hans), (2, 3, TokenType.hans),
          (4, 4, TokenType.punc)]
    pt = _form_parsed_tokens(pt)
    assert index_last_end_of_word(pt) == 4
    pt = [(0, 3, TokenType.hans), (4, 4, TokenType.hans),
          (7, 10, TokenType.punc), (13, 17, TokenType.space)]
    pt = _form_parsed_tokens(pt)
    assert index_last_end_of_word(pt) == 10
    pt = [(0, 0, TokenType.punc), (3, 3, TokenType.punc)]
    pt = _form_parsed_tokens(pt)
    assert index_last_end_of_word(pt) == 3


def test_index_prev_end_of_word():
    pt = []
    assert index_prev_end_of_word(pt, 0) is None
    pt = [(0, 3, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_prev_end_of_word(pt, 0) is None
    assert index_prev_end_of_word(pt, 1) is None
    assert index_prev_end_of_word(pt, 3) is None
    pt = [(0, 3, TokenType.hans), (4, 5, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_prev_end_of_word(pt, 0) is None
    assert index_prev_end_of_word(pt, 1) is None
    assert index_prev_end_of_word(pt, 3) is None
    assert index_prev_end_of_word(pt, 4) == 3
    assert index_prev_end_of_word(pt, 5) == 3
    pt = [(0, 3, TokenType.hans), (6, 9, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_prev_end_of_word(pt, 0) is None
    assert index_prev_end_of_word(pt, 1) is None
    assert index_prev_end_of_word(pt, 3) is None
    assert index_prev_end_of_word(pt, 4) is None
    assert index_prev_end_of_word(pt, 5) is None
    assert index_prev_end_of_word(pt, 6) == 3
    assert index_prev_end_of_word(pt, 7) == 3
    assert index_prev_end_of_word(pt, 9) == 3
    pt = [(0, 3, TokenType.hans), (4, 4, TokenType.punc),
          (5, 6, TokenType.space)]
    pt = _form_parsed_tokens(pt)
    assert index_prev_end_of_word(pt, 5) == 4
    assert index_prev_end_of_word(pt, 6) == 4
    pt = [(0, 1, TokenType.space), (2, 2, TokenType.hans),
          (5, 6, TokenType.punc), (8, 9, TokenType.space)]
    pt = _form_parsed_tokens(pt)
    assert index_prev_end_of_word(pt, 0) is None
    assert index_prev_end_of_word(pt, 1) is None
    assert index_prev_end_of_word(pt, 2) is None
    assert index_prev_end_of_word(pt, 3) is None
    assert index_prev_end_of_word(pt, 4) is None
    assert index_prev_end_of_word(pt, 5) == 2
    assert index_prev_end_of_word(pt, 6) == 2
    assert index_prev_end_of_word(pt, 7) == 2
    assert index_prev_end_of_word(pt, 8) == 6
    assert index_prev_end_of_word(pt, 9) == 6
    pt = [(0, 0, TokenType.punc), (3, 3, TokenType.punc)]
    pt = _form_parsed_tokens(pt)
    assert index_prev_end_of_word(pt, 4) == 0


def test_index_last_end_of_WORD():
    pt = []
    assert index_last_end_of_WORD(pt) == 0
    pt = [(0, 1, TokenType.hans), (2, 3, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_last_end_of_WORD(pt) == 3
    pt = [(0, 3, TokenType.hans), (4, 4, TokenType.hans),
          (7, 10, TokenType.punc), (13, 17, TokenType.space)]
    pt = _form_parsed_tokens(pt)
    assert index_last_end_of_WORD(pt) == 10
    pt = [(0, 3, TokenType.hans), (4, 4, TokenType.punc),
          (7, 10, TokenType.hans), (13, 17, TokenType.space)]
    pt = _form_parsed_tokens(pt)
    assert index_last_end_of_WORD(pt) == 10
    pt = [(0, 1, TokenType.space), (2, 5, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_last_end_of_WORD(pt) == 5
    pt = [(0, 1, TokenType.space), (2, 5, TokenType.hans),
          (8, 8, TokenType.space)]
    pt = _form_parsed_tokens(pt)
    assert index_last_end_of_WORD(pt) == 5
    pt = [(0, 0, TokenType.punc), (3, 3, TokenType.punc)]
    pt = _form_parsed_tokens(pt)
    assert index_last_end_of_WORD(pt) == 3


def test_index_prev_end_of_WORD():
    pt = []
    assert index_prev_end_of_WORD(pt, 0) is None
    pt = [(0, 1, TokenType.space), (2, 3, TokenType.hans),
          (5, 5, TokenType.punc), (8, 9, TokenType.space),
          (10, 10, TokenType.punc), (13, 13, TokenType.hans),
          (16, 16, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_prev_end_of_WORD(pt, 0) is None
    assert index_prev_end_of_WORD(pt, 1) is None
    assert index_prev_end_of_WORD(pt, 2) is None
    assert index_prev_end_of_WORD(pt, 3) is None
    assert index_prev_end_of_WORD(pt, 5) is None
    assert index_prev_end_of_WORD(pt, 6) is None
    assert index_prev_end_of_WORD(pt, 7) is None
    assert index_prev_end_of_WORD(pt, 8) == 5
    assert index_prev_end_of_WORD(pt, 9) == 5
    assert index_prev_end_of_WORD(pt, 10) == 5
    assert index_prev_end_of_WORD(pt, 11) == 5
    assert index_prev_end_of_WORD(pt, 13) == 10
    assert index_prev_end_of_WORD(pt, 14) == 10
    assert index_prev_end_of_WORD(pt, 15) == 10
    assert index_prev_end_of_WORD(pt, 16) == 13
    assert index_prev_end_of_WORD(pt, 17) == 13
    pt = [(0, 0, TokenType.punc), (3, 3, TokenType.punc)]
    pt = _form_parsed_tokens(pt)
    assert index_prev_end_of_WORD(pt, 4) == 0


def test_index_first_start_of_word():
    pt = []
    assert index_first_start_of_word(pt) == 0
    pt = [(0, 1, TokenType.hans), (2, 3, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_first_start_of_word(pt) == 0
    pt = [(0, 2, TokenType.space), (3, 6, TokenType.punc),
          (9, 12, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_first_start_of_word(pt) == 3
    pt = [(0, 0, TokenType.punc), (3, 3, TokenType.punc)]
    pt = _form_parsed_tokens(pt)
    assert index_first_start_of_word(pt) == 0


def test_index_next_start_of_word():
    pt = []
    assert index_next_start_of_word(pt, 0) is None
    pt = [(0, 3, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_next_start_of_word(pt, 0) is None
    assert index_next_start_of_word(pt, 1) is None
    assert index_next_start_of_word(pt, 3) is None
    pt = [(0, 3, TokenType.hans), (4, 5, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_next_start_of_word(pt, 0) == 4
    assert index_next_start_of_word(pt, 1) == 4
    assert index_next_start_of_word(pt, 3) == 4
    assert index_next_start_of_word(pt, 4) is None
    assert index_next_start_of_word(pt, 5) is None
    pt = [(0, 3, TokenType.hans), (6, 9, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_next_start_of_word(pt, 0) == 6
    assert index_next_start_of_word(pt, 1) == 6
    assert index_next_start_of_word(pt, 3) == 6
    assert index_next_start_of_word(pt, 4) == 6
    assert index_next_start_of_word(pt, 5) == 6
    assert index_next_start_of_word(pt, 6) is None
    assert index_next_start_of_word(pt, 7) is None
    assert index_next_start_of_word(pt, 9) is None
    pt = [(0, 2, TokenType.space), (3, 6, TokenType.hans),
          (7, 7, TokenType.punc)]
    pt = _form_parsed_tokens(pt)
    assert index_next_start_of_word(pt, 0) == 3
    assert index_next_start_of_word(pt, 2) == 3
    assert index_next_start_of_word(pt, 3) == 7
    assert index_next_start_of_word(pt, 6) == 7
    assert index_next_start_of_word(pt, 7) is None
    pt = [(0, 1, TokenType.space), (2, 3, TokenType.punc),
          (5, 6, TokenType.hans), (8, 9, TokenType.space)]
    pt = _form_parsed_tokens(pt)
    assert index_next_start_of_word(pt, 0) == 2
    assert index_next_start_of_word(pt, 1) == 2
    assert index_next_start_of_word(pt, 2) == 5
    assert index_next_start_of_word(pt, 3) == 5
    assert index_next_start_of_word(pt, 4) == 5
    assert index_next_start_of_word(pt, 5) is None
    assert index_next_start_of_word(pt, 6) is None
    assert index_next_start_of_word(pt, 7) is None
    assert index_next_start_of_word(pt, 8) is None
    assert index_next_start_of_word(pt, 9) is None
    pt = [(0, 0, TokenType.punc), (3, 3, TokenType.punc)]
    pt = _form_parsed_tokens(pt)
    assert index_next_start_of_word(pt, 0) == 3
    assert index_next_start_of_word(pt, 1) == 3
    assert index_next_start_of_word(pt, 2) == 3
    assert index_next_start_of_word(pt, 3) is None


def test_index_first_start_of_WORD():
    pt = []
    assert index_first_start_of_WORD(pt) == 0
    pt = [(0, 1, TokenType.hans), (2, 3, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_first_start_of_WORD(pt) == 0
    pt = [(0, 2, TokenType.space), (3, 6, TokenType.punc),
          (9, 12, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_first_start_of_WORD(pt) == 3
    pt = [(0, 0, TokenType.punc), (3, 3, TokenType.punc)]
    pt = _form_parsed_tokens(pt)
    assert index_first_start_of_WORD(pt) == 0


def test_index_next_start_of_WORD():
    pt = []
    assert index_next_start_of_WORD(pt, 0) is None
    pt = [(0, 3, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_next_start_of_WORD(pt, 0) is None
    assert index_next_start_of_WORD(pt, 1) is None
    assert index_next_start_of_WORD(pt, 3) is None
    pt = [(0, 3, TokenType.hans), (4, 5, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_next_start_of_WORD(pt, 0) == 4
    assert index_next_start_of_WORD(pt, 1) == 4
    assert index_next_start_of_WORD(pt, 3) == 4
    assert index_next_start_of_WORD(pt, 4) is None
    assert index_next_start_of_WORD(pt, 5) is None
    pt = [(0, 3, TokenType.hans), (6, 9, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_next_start_of_WORD(pt, 0) == 6
    assert index_next_start_of_WORD(pt, 1) == 6
    assert index_next_start_of_WORD(pt, 3) == 6
    assert index_next_start_of_WORD(pt, 4) == 6
    assert index_next_start_of_WORD(pt, 5) == 6
    assert index_next_start_of_WORD(pt, 6) is None
    assert index_next_start_of_WORD(pt, 7) is None
    assert index_next_start_of_WORD(pt, 9) is None
    pt = [(0, 2, TokenType.space), (3, 6, TokenType.hans),
          (7, 7, TokenType.punc)]
    pt = _form_parsed_tokens(pt)
    assert index_next_start_of_WORD(pt, 0) == 3
    assert index_next_start_of_WORD(pt, 2) == 3
    assert index_next_start_of_WORD(pt, 3) is None
    assert index_next_start_of_WORD(pt, 6) is None
    assert index_next_start_of_WORD(pt, 7) is None
    pt = [(0, 1, TokenType.space), (2, 3, TokenType.punc),
          (5, 6, TokenType.hans), (8, 9, TokenType.space)]
    pt = _form_parsed_tokens(pt)
    assert index_next_start_of_WORD(pt, 0) == 2
    assert index_next_start_of_WORD(pt, 1) == 2
    assert index_next_start_of_WORD(pt, 2) == 5
    assert index_next_start_of_WORD(pt, 3) == 5
    assert index_next_start_of_WORD(pt, 4) == 5
    assert index_next_start_of_WORD(pt, 5) is None
    assert index_next_start_of_WORD(pt, 6) is None
    assert index_next_start_of_WORD(pt, 7) is None
    assert index_next_start_of_WORD(pt, 8) is None
    assert index_next_start_of_WORD(pt, 9) is None
    pt = [(0, 0, TokenType.punc), (3, 3, TokenType.punc)]
    pt = _form_parsed_tokens(pt)
    assert index_next_start_of_WORD(pt, 0) == 3
    assert index_next_start_of_WORD(pt, 1) == 3
    assert index_next_start_of_WORD(pt, 2) == 3
    assert index_next_start_of_WORD(pt, 3) is None


def test_index_first_end_of_word():
    pt = []
    assert index_first_end_of_word(pt) == 0
    pt = [(0, 1, TokenType.hans), (2, 3, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_first_end_of_word(pt) == 1
    pt = [(0, 2, TokenType.space), (3, 6, TokenType.punc),
          (9, 12, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_first_end_of_word(pt) == 6
    pt = [(0, 1, TokenType.punc), (3, 3, TokenType.punc)]
    pt = _form_parsed_tokens(pt)
    assert index_first_end_of_word(pt) == 1


def test_index_next_end_of_word():
    pt = []
    assert index_next_end_of_word(pt, 0) is None
    pt = [(0, 3, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_next_end_of_word(pt, 0) == 3
    assert index_next_end_of_word(pt, 1) == 3
    assert index_next_end_of_word(pt, 2) == 3
    assert index_next_end_of_word(pt, 3) is None
    pt = [(0, 3, TokenType.hans), (6, 9, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_next_end_of_word(pt, 0) == 3
    assert index_next_end_of_word(pt, 1) == 3
    assert index_next_end_of_word(pt, 3) == 9
    assert index_next_end_of_word(pt, 6) == 9
    assert index_next_end_of_word(pt, 7) == 9
    assert index_next_end_of_word(pt, 8) == 9
    assert index_next_end_of_word(pt, 9) is None
    pt = [(0, 2, TokenType.space), (3, 6, TokenType.hans),
          (7, 7, TokenType.punc)]
    pt = _form_parsed_tokens(pt)
    assert index_next_end_of_word(pt, 0) == 6
    assert index_next_end_of_word(pt, 2) == 6
    assert index_next_end_of_word(pt, 3) == 6
    assert index_next_end_of_word(pt, 6) == 7
    assert index_next_end_of_word(pt, 7) is None
    pt = [(0, 1, TokenType.space), (2, 3, TokenType.punc),
          (5, 6, TokenType.hans), (8, 9, TokenType.space)]
    pt = _form_parsed_tokens(pt)
    assert index_next_end_of_word(pt, 0) == 3
    assert index_next_end_of_word(pt, 1) == 3
    assert index_next_end_of_word(pt, 2) == 3
    assert index_next_end_of_word(pt, 3) == 6
    assert index_next_end_of_word(pt, 5) == 6
    assert index_next_end_of_word(pt, 6) is None
    assert index_next_end_of_word(pt, 8) is None
    assert index_next_end_of_word(pt, 9) is None
    pt = [(0, 0, TokenType.punc), (3, 3, TokenType.punc)]
    pt = _form_parsed_tokens(pt)
    assert index_next_end_of_word(pt, 0) == 3
    assert index_next_end_of_word(pt, 3) is None


def test_index_first_end_of_WORD():
    pt = []
    assert index_first_end_of_WORD(pt) == 0
    pt = [(0, 1, TokenType.hans), (2, 3, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_first_end_of_WORD(pt) == 1
    pt = [(0, 2, TokenType.space), (3, 6, TokenType.punc),
          (9, 12, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_first_end_of_WORD(pt) == 6
    pt = [(0, 2, TokenType.space), (3, 6, TokenType.hans),
          (9, 12, TokenType.punc)]
    pt = _form_parsed_tokens(pt)
    assert index_first_end_of_WORD(pt) == 12
    pt = [(0, 1, TokenType.punc), (3, 3, TokenType.punc)]
    pt = _form_parsed_tokens(pt)
    assert index_first_end_of_WORD(pt) == 1


def test_index_next_end_of_WORD():
    pt = []
    assert index_next_end_of_WORD(pt, 0) is None
    pt = [(0, 3, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_next_end_of_WORD(pt, 0) == 3
    assert index_next_end_of_WORD(pt, 1) == 3
    assert index_next_end_of_WORD(pt, 2) == 3
    assert index_next_end_of_WORD(pt, 3) is None
    pt = [(0, 3, TokenType.hans), (6, 9, TokenType.hans)]
    pt = _form_parsed_tokens(pt)
    assert index_next_end_of_WORD(pt, 0) == 3
    assert index_next_end_of_WORD(pt, 1) == 3
    assert index_next_end_of_WORD(pt, 3) == 9
    assert index_next_end_of_WORD(pt, 6) == 9
    assert index_next_end_of_WORD(pt, 7) == 9
    assert index_next_end_of_WORD(pt, 8) == 9
    assert index_next_end_of_WORD(pt, 9) is None
    pt = [(0, 2, TokenType.space), (3, 6, TokenType.hans),
          (7, 7, TokenType.punc)]
    pt = _form_parsed_tokens(pt)
    assert index_next_end_of_WORD(pt, 0) == 7
    assert index_next_end_of_WORD(pt, 2) == 7
    assert index_next_end_of_WORD(pt, 3) == 7
    assert index_next_end_of_WORD(pt, 6) == 7
    assert index_next_end_of_WORD(pt, 7) is None
    pt = [(0, 1, TokenType.space), (2, 3, TokenType.punc),
          (5, 6, TokenType.hans), (8, 9, TokenType.space)]
    pt = _form_parsed_tokens(pt)
    assert index_next_end_of_WORD(pt, 0) == 3
    assert index_next_end_of_WORD(pt, 1) == 3
    assert index_next_end_of_WORD(pt, 2) == 3
    assert index_next_end_of_WORD(pt, 3) == 6
    assert index_next_end_of_WORD(pt, 5) == 6
    assert index_next_end_of_WORD(pt, 6) is None
    assert index_next_end_of_WORD(pt, 8) is None
    assert index_next_end_of_WORD(pt, 9) is None
    pt = [(0, 0, TokenType.punc), (3, 3, TokenType.punc)]
    pt = _form_parsed_tokens(pt)
    assert index_next_end_of_WORD(pt, 0) == 3
    assert index_next_end_of_WORD(pt, 3) is None
