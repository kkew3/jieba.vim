# Copyright 2024-2025 Kaiwen Wu. All Rights Reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License. You may obtain a copy
# of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
# WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
# License for the specific language governing permissions and limitations
# under the License.

import vim  # type: ignore

PREVIEW_MAX_LIMIT = 99999


def preview_cancel():
    vim.command('hi clear JiebaPreview')


def get_preview_limit():
    """
    g:jieba_vim_preview_limits (int) is used to set the number of positions to
    highlight. When positive, preview that many positions. When zero, preview
    as many positions but limited to current line. When negative, prevew at
    most ``preview_max_limit`` (99999) positions. Default to zero.
    """
    try:
        limit = int(vim.eval('get(g:, "jieba_vim_preview_limits", 0)'))
    except ValueError:
        limit = 0
    if limit < 0:
        limit = PREVIEW_MAX_LIMIT
    return min(limit, PREVIEW_MAX_LIMIT)


def preview(preview_func):
    """
    Preview corresponding navigation.

    :param preview_func: a function from ``jieba_vim.jieba_vim_rs`` module of
           signature ``(buffer, cursor_pos, limit) -> list[cursor_pos]``.
    """
    vim.command('hi link JiebaPreview IncSearch')
    limit = get_preview_limit()
    cursor_positions = preview_func(
        vim.current.buffer, vim.current.window.cursor, limit
    )
    if cursor_positions:
        # build match pattern if there's any positions to highlight
        match_pat = '|'.join(
            '%{}c%{}l'.format(col + 1, row) for row, col in cursor_positions
        )
        vim.command('match JiebaPreview /\\v{}/'.format(match_pat))
    else:
        preview_cancel()
