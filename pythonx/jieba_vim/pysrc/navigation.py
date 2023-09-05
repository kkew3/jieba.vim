"""
Three types of tokens (see ``get_token_type`` function)::

    - ``space`` (S for short): whitespace characters
    - ``punc`` (P for short): Chinese punctuation
    - ``non_word`` (N for short): other non-word characters
    - ``hans`` (H for short): everything else (including alphanum)

Implicit spaces are inserted between various types of tokens. For example,
between H and H there will be an implicit space to form words in Chinese.
For details of the insertion rule, please refer to
``insert_implicit_space_rule``.

After insertion of implicit space, we may treat Chinese text as if the words
were separated by space like English. We define 'word' as a non-space token,
and 'WORD' as a successive sequence of non-space tokens.

Motions::

    - ``b``: jumps backward to each start of word
    - ``ge``: jumps backward to each end of word
    - ``w``: jumps forward to each start of word
    - ``e``: jumps forward to each end of word
    - ``B``: jumps backward to each start of WORD
    - ``gE``: jumps backward to each end of WORD
    - ``W``: jumps forward to each start of WORD
    - ``E``: jumps forward to each end of WORD
"""
import re
import collections
import functools

import jieba_vim
from . import punc

pat_space = re.compile(r'\s+')
pat_nonword = re.compile(r'\W+')
pat_punc = re.compile('[' + punc.punctuation + ']+')


class TokenType:
    space = 1
    punc = 2
    hans = 3
    non_word = 4


def get_token_type(token):
    """
    Decide the type of a token.

    :type token: str
    :return: an instance of ``TokenType``
    :rtype: int
    """
    if not token or pat_space.fullmatch(token):
        return TokenType.space
    if pat_punc.fullmatch(token):
        return TokenType.punc
    if pat_nonword.fullmatch(token):
        return TokenType.non_word
    return TokenType.hans


ParsedToken = collections.namedtuple('ParsedToken', ['i', 'j', 't'])


def parse_tokens(tokens):
    """
    Parse each token as a tuple ``(i, j, t)`` such that ``i`` denotes the
    byte index of the first character of the token, ``j`` denotes the
    byte index of the last character of the token, ``t`` denotes the
    type of the token. If ``j`` is less than ``i``, it means that the
    underlying token is an empty string.
    """
    cum_l = 0
    parsed = []
    for tok in tokens:
        i = cum_l
        t = get_token_type(tok)
        cum_l += len(tok.encode('utf-8'))
        j = cum_l - len(tok[-1].encode('utf-8'))
        parsed.append(ParsedToken(i, j, t))
    return parsed


def stack_merge(elements, rule_func):
    """
    Merge and transform ``elements`` according to ``rule_func`` by
    pushing them itertatively to a stack.

    ``rule_func`` should be a callable that takes ``(e1, e2)`` as argument
    and returns a list of transformed elements or ``None``. ``e1`` may be
    ``None`` when ``e2`` is the first element of ``elements``.
    """
    stack = []
    for pt in elements:
        trans_pt_list = rule_func(stack[-1] if stack else None, pt)
        if trans_pt_list is None:
            stack.append(pt)
        elif trans_pt_list[0] is None:
            del trans_pt_list[0]
            stack.extend(trans_pt_list)
        else:
            del stack[-1]
            stack.extend(trans_pt_list)
    return stack


def _gen_implicit_space_in_between(parsed_tok2):
    i2 = parsed_tok2.i
    return ParsedToken(i2, i2 - 1, TokenType.space)


def insert_implicit_space_rule(parsed_tok1, parsed_tok2):
    if parsed_tok1 is None:
        return None
    to_insert_table = {
        (TokenType.hans, TokenType.hans): True,
        (TokenType.hans, TokenType.punc): False,
        (TokenType.hans, TokenType.space): False,
        (TokenType.hans, TokenType.non_word): False,
        (TokenType.punc, TokenType.hans): True,
        (TokenType.punc, TokenType.punc): True,
        (TokenType.punc, TokenType.space): False,
        (TokenType.punc, TokenType.non_word): True,
        (TokenType.space, TokenType.hans): False,
        (TokenType.space, TokenType.punc): False,
        (TokenType.space, TokenType.space): False,
        (TokenType.space, TokenType.non_word): False,
        (TokenType.non_word, TokenType.hans): False,
        (TokenType.non_word, TokenType.punc): False,
        (TokenType.non_word, TokenType.space): False,
        (TokenType.non_word, TokenType.non_word): False,
    }
    if to_insert_table[parsed_tok1.t, parsed_tok2.t]:
        imp_space = _gen_implicit_space_in_between(parsed_tok2)
        return [parsed_tok1, imp_space, parsed_tok2]
    return None


def index_tokens(parsed_tokens, bi):
    """
    Returns the token index at which the byte index ``bi`` lies.

    :param parsed_tokens: a list of ``ParsedToken``
    :type tokens: list
    :param bi: a byte index
    """
    # The reason to traverse `parsed_tokens` in reversed order is that we need
    # to index after all implicit space tokens.
    for ti in reversed(range(len(parsed_tokens))):
        if parsed_tokens[ti].i <= bi:
            return ti
    raise IndexError(('token index of byte index `{}` not found in '
                      'parsed tokens `{}`').format(bi, parsed_tokens))


def index_last_start_of_word(parsed_tokens):
    if not parsed_tokens:
        return 0
    for ti in reversed(range(len(parsed_tokens))):
        if parsed_tokens[ti].t != TokenType.space:
            return parsed_tokens[ti].i
    return None


def index_prev_start_of_word(parsed_tokens, ci):
    if not parsed_tokens:
        return None
    ti = index_tokens(parsed_tokens, ci)
    if ci == parsed_tokens[ti].i:
        ti -= 1
    while ti >= 0:
        if parsed_tokens[ti].t != TokenType.space:
            return parsed_tokens[ti].i
        ti -= 1
    return None


def index_last_start_of_WORD(parsed_tokens):
    if not parsed_tokens:
        return 0
    last_valid_i = None
    for ti in reversed(range(len(parsed_tokens))):
        if parsed_tokens[ti].t != TokenType.space:
            last_valid_i = parsed_tokens[ti].i
        elif last_valid_i is not None:
            break
    return last_valid_i


def index_prev_start_of_WORD(parsed_tokens, ci):
    if not parsed_tokens:
        return None
    ti = index_tokens(parsed_tokens, ci)
    if ci == parsed_tokens[ti].i:
        ti -= 1
    last_valid_i = None
    while ti >= 0:
        if parsed_tokens[ti].t != TokenType.space:
            last_valid_i = parsed_tokens[ti].i
        elif last_valid_i is not None:
            break
        ti -= 1
    return last_valid_i


def index_last_end_of_word(parsed_tokens):
    if not parsed_tokens:
        return 0
    for ti in reversed(range(len(parsed_tokens))):
        if parsed_tokens[ti].t != TokenType.space:
            return parsed_tokens[ti].j
    return None


def index_prev_end_of_word(parsed_tokens, ci):
    if not parsed_tokens:
        return None
    ti = index_tokens(parsed_tokens, ci) - 1
    while ti >= 0:
        if parsed_tokens[ti].t != TokenType.space:
            return parsed_tokens[ti].j
        ti -= 1
    return None


def index_last_end_of_WORD(parsed_tokens):
    if not parsed_tokens:
        return 0
    for ti in reversed(range(len(parsed_tokens))):
        if parsed_tokens[ti].t != TokenType.space:
            return parsed_tokens[ti].j
    return None


def index_prev_end_of_WORD(parsed_tokens, ci):
    if not parsed_tokens:
        return None
    ti = index_tokens(parsed_tokens, ci)
    if parsed_tokens[ti].t == TokenType.space:
        ti -= 1
    else:
        while ti >= 0 and parsed_tokens[ti].t != TokenType.space:
            ti -= 1
    while ti >= 0:
        if parsed_tokens[ti].t != TokenType.space:
            return parsed_tokens[ti].j
        ti -= 1
    return None


def index_first_start_of_word(parsed_tokens):
    if not parsed_tokens:
        return 0
    for tok in parsed_tokens:
        if tok.t != TokenType.space:
            return tok.i
    return None


def index_next_start_of_word(parsed_tokens, ci):
    if not parsed_tokens:
        return None
    ti = index_tokens(parsed_tokens, ci) + 1
    while ti < len(parsed_tokens):
        if parsed_tokens[ti].t != TokenType.space:
            return parsed_tokens[ti].i
        ti += 1
    return None


def index_first_start_of_WORD(parsed_tokens):
    if not parsed_tokens:
        return 0
    for tok in parsed_tokens:
        if tok.t != TokenType.space:
            return tok.i
    return None


def index_next_start_of_WORD(parsed_tokens, ci):
    if not parsed_tokens:
        return None
    ti = index_tokens(parsed_tokens, ci)
    if parsed_tokens[ti].t == TokenType.space:
        ti += 1
    else:
        while (ti < len(parsed_tokens)
               and parsed_tokens[ti].t != TokenType.space):
            ti += 1
    while ti < len(parsed_tokens):
        if parsed_tokens[ti].t != TokenType.space:
            return parsed_tokens[ti].i
        ti += 1
    return None


def index_first_end_of_word(parsed_tokens):
    if not parsed_tokens:
        return 0
    for tok in parsed_tokens:
        if tok.t != TokenType.space:
            return tok.j
    return None


def index_next_end_of_word(parsed_tokens, ci):
    if not parsed_tokens:
        return None
    ti = index_tokens(parsed_tokens, ci)
    if ci == parsed_tokens[ti].j:
        ti += 1
    while ti < len(parsed_tokens):
        if parsed_tokens[ti].t != TokenType.space:
            return parsed_tokens[ti].j
        ti += 1
    return None


def index_first_end_of_WORD(parsed_tokens):
    if not parsed_tokens:
        return 0
    last_valid_j = None
    for tok in parsed_tokens:
        if tok.t != TokenType.space:
            last_valid_j = tok.j
        elif last_valid_j is not None:
            break
    return last_valid_j


def index_next_end_of_WORD(parsed_tokens, ci):
    if not parsed_tokens:
        return None
    ti = index_tokens(parsed_tokens, ci)
    if ci == parsed_tokens[ti].j:
        ti += 1
    last_valid_j = None
    while ti < len(parsed_tokens):
        if parsed_tokens[ti].t != TokenType.space:
            last_valid_j = parsed_tokens[ti].j
        elif last_valid_j is not None:
            break
        ti += 1
    return last_valid_j


def _navigate(primary_index_func, secondary_index_func, backward, buffer,
              cursor_pos):
    """
    :param primary_index_func: the index function invoked on the first attempt
    :param secondary_index_func: the index function invoked on the second
           attempt
    :param backward: whether the two index function go backward or not
    :param buffer: current buffer, a list of lines
    :param cursor_col: the (row, col) tuple of the cursor
    :return: the new cursor position
    """
    if backward:
        sentinel_row = 1
        row_step = -1
    else:
        sentinel_row = len(buffer)
        row_step = 1
    row, col = cursor_pos
    if row == sentinel_row:
        pt = parse_tokens(jieba_vim.jieba_cut(buffer[row - 1]))
        pt = stack_merge(pt, insert_implicit_space_rule)
        col = primary_index_func(pt, col)
        if col is None:
            if backward:
                col = pt[0].i if pt else 0
            else:
                col = pt[-1].j if pt else 0
        return row, col
    pt = parse_tokens(jieba_vim.jieba_cut(buffer[row - 1]))
    pt = stack_merge(pt, insert_implicit_space_rule)
    col = primary_index_func(pt, col)
    if col is not None:
        return row, col
    row += row_step
    while row != sentinel_row:
        pt = parse_tokens(jieba_vim.jieba_cut(buffer[row - 1]))
        pt = stack_merge(pt, insert_implicit_space_rule)
        col = secondary_index_func(pt)
        if col is not None:
            return row, col
        row += row_step
    pt = parse_tokens(jieba_vim.jieba_cut(buffer[row - 1]))
    pt = stack_merge(pt, insert_implicit_space_rule)
    col = secondary_index_func(pt)
    if col is None:
        if backward:
            col = pt[0].i if pt else 0
        else:
            col = pt[-1].j if pt else 0
    return row, col


wordmotion_b = functools.partial(_navigate, index_prev_start_of_word,
                                 index_last_start_of_word, True)
wordmotion_B = functools.partial(_navigate, index_prev_start_of_WORD,
                                 index_last_start_of_WORD, True)
wordmotion_ge = functools.partial(_navigate, index_prev_end_of_word,
                                  index_last_end_of_word, True)
wordmotion_gE = functools.partial(_navigate, index_prev_end_of_WORD,
                                  index_last_end_of_WORD, True)
wordmotion_w = functools.partial(_navigate, index_next_start_of_word,
                                 index_first_start_of_word, False)
wordmotion_W = functools.partial(_navigate, index_next_start_of_WORD,
                                 index_first_start_of_WORD, False)
wordmotion_e = functools.partial(_navigate, index_next_end_of_word,
                                 index_first_end_of_word, False)
wordmotion_E = functools.partial(_navigate, index_next_end_of_WORD,
                                 index_first_end_of_WORD, False)
