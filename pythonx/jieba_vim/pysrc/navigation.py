"""
Three types of tokens (see ``get_token_type`` function)::

    - ``space`` (S for short): whitespace characters
    - ``punc`` (P for short): Chinese punctuation
    - ``hans`` (H for short): everything else (including alphanum)

Between every H, if there's no S, an implicit S will be inserted.

Motions::

    - ``backward_start``: jumps backward to each start of P or H
    - ``backward_end``: similar
    - ``forward_start``: similar
    - ``forward_end``: similar
    - ``big_backward_start``: jumps backward to each start of non-S
    - ``big_backward_end``: similar
    - ``big_forward_start``: similar
    - ``big_forward_end``: similar

Difference between "P or H" and "non-S":
For example, a sequence of P or H is considered *one* block of non-S.
"""
import re
import collections

import jieba_vim
from . import punc

pat_space = re.compile(r'\s+')
pat_punc = re.compile('[' + punc.punctuation + ']+')


class TokenType:
    space = 1
    punc = 2
    hans = 3


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


def index_tokens(parsed_tokens, bi):
    """
    Returns the token index at which the byte index ``bi`` lies.

    :param parsed_tokens: a list of ``ParsedToken``
    :type tokens: list
    :param bi: a byte index
    """
    for ti in reversed(range(len(parsed_tokens))):
        if parsed_tokens[ti].i <= bi:
            return ti
    raise IndexError(('token index of byte index `{}` not found in '
                      'parsed tokens `{}`').format(bi, parsed_tokens))


def index_last_start_of_PorH(parsed_tokens):
    if not parsed_tokens:
        return 0
    for ti in reversed(range(len(parsed_tokens))):
        if parsed_tokens[ti].t != TokenType.space:
            return parsed_tokens[ti].i
    return None


def index_prev_start_of_PorH(parsed_tokens, ci):
    # if current character index is zero, no pervious start can be found in
    # current line of tokens
    if ci == 0:
        return None
    # here we assume that when ci > 0, parsed_tokens is nonempty
    ti = index_tokens(parsed_tokens, ci)
    if ci == parsed_tokens[ti].i:
        ti -= 1
    while ti >= 0:
        if parsed_tokens[ti].t != TokenType.space:
            return parsed_tokens[ti].i
        ti -= 1
    return None


def backward_word_start(buffer, cursor_pos):
    """
    :param buffer: current buffer, a list of lines
    :param cursor_col: the (row, col) tuple of the cursor
    :return: the new cursor position
    """
    row, col = cursor_pos
    if row == 1:
        pt = parse_tokens(jieba_vim.jieba_cut(buffer[row - 1]))
        col = index_prev_start_of_PorH(pt, col)
        if col is None:
            col = 0
        return row, col
    pt = parse_tokens(jieba_vim.jieba_cut(buffer[row - 1]))
    col = index_prev_start_of_PorH(pt, col)
    if col is not None:
        return row, col
    row -= 1
    while row != 1:
        pt = parse_tokens(jieba_vim.jieba_cut(buffer[row - 1]))
        col = index_last_start_of_PorH(pt)
        if col is not None:
            return row, col
        row -= 1
    # if reached here, row == 1
    pt = parse_tokens(jieba_vim.jieba_cut(buffer[row - 1]))
    col = index_last_start_of_PorH(pt)
    if col is None:
        col = 0
    return row, col
