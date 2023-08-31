import os
import logging
import functools

import vim

from . import libjieba


def jieba_initialize():
    default_dict_path = os.path.join(
        os.path.dirname(libjieba.__file__), 'dict.txt')
    user_dict_path = vim.eval('get(g:, "jieba_vim_user_dict", "")')
    libjieba.setLogLevel(logging.WARNING)
    if user_dict_path:
        try:
            libjieba.initialize(dictionary=user_dict_path)
        except FileNotFoundError:
            libjieba.initialize(dictionary=default_dict_path)
            vim.command(
                'echoerr "path specified by g:jieba_vim_user_dict not found; '
                'using default dict"')
    else:
        libjieba.initialize()
    vim.command('let g:jieba_vim_initialized = 1')


def jieba_initialized(func):
    """Decorator to ensure ``jieba_initialize`` has been called."""
    @functools.wraps(func)
    def _wrapper(*args, **kwargs):
        if not int(vim.eval('exists("g:jieba_vim_initialized")')):
            jieba_initialize()
        return func(*args, **kwargs)

    return _wrapper
