import vim

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


def preview(navi_func):
    """
    Preview corresponding navigation.

    :param navi_func: a function from ``jieba_vim.jieba_navi_rs`` module of
           signature ``(buffer, cursor_pos) -> cursor_pos``.
    """
    vim.command('hi link JiebaPreview IncSearch')
    limit = get_preview_limit()
    curr_row, curr_col = vim.current.window.cursor
    b = vim.current.buffer
    cursor_positions = []
    if limit == 0:
        while True:
            next_row, next_col = navi_func(b, (curr_row, curr_col))
            # reaches either beginning of file or end of file
            if (next_row, next_col) == (curr_row, curr_col):
                break
            # reaches either previous line or next line
            if next_row != curr_row:
                break
            cursor_positions.append((next_row, next_col))
            curr_row, curr_col = next_row, next_col
    else:
        while len(cursor_positions) < limit:
            next_row, next_col = navi_func(b, (curr_row, curr_col))
            # reaches either beginning of file or end of file
            if (next_row, next_col) == (curr_row, curr_col):
                break
            cursor_positions.append((next_row, next_col))
            curr_row, curr_col = next_row, next_col
    if cursor_positions:
        # build match pattern if there's any positions to highlight
        match_pat = '|'.join('%{}c%{}l'.format(col + 1, row)
                             for row, col in cursor_positions)
        vim.command('match JiebaPreview /\\v{}/'.format(match_pat))
    else:
        preview_cancel()
