"""
These names are dynamically defined in this module::

    - forward_word_start
    - forward_word_start_v
    - forward_word_end
    - forward_word_end_v
    - forward_WORD_start
    - forward_WORD_start_v
    - forward_WORD_end
    - forward_WORD_end_v
    - backward_word_start
    - backward_word_start_v
    - backward_word_end
    - backward_word_end_v
    - backward_WORD_start
    - backward_WORD_start_v
    - backward_WORD_end
    - backward_WORD_end_v
"""
import itertools

import vim

from .pysrc import navigation
from .init import jieba_initialized


def vim_wrapper_factory(py_navi_func_name):
    py_navi_func = getattr(navigation, py_navi_func_name)

    @jieba_initialized
    def _wrapper():
        for _ in range(int(vim.eval('v:count1'))):
            vim.current.window.cursor = py_navi_func(vim.current.buffer,
                                                     vim.current.window.cursor)

    # known issue: responding_to_vcount won't work in visual mode
    @jieba_initialized
    def _wrapper_v():
        # Reference: https://stackoverflow.com/q/16212801/7881370
        vim.command('normal! gv')
        vim.current.window.cursor = py_navi_func(vim.current.buffer,
                                                 vim.current.window.cursor)

    return {py_navi_func_name: _wrapper, py_navi_func_name + '_v': _wrapper_v}


def get_navi_func_names():
    func_names = []
    dir_opts = ['forward', 'backward']
    word_opts = ['word', 'WORD']
    end_opts = ['start', 'end']
    for opt in itertools.product(dir_opts, word_opts, end_opts):
        func_names.append('_'.join(opt))
    return func_names


for name in get_navi_func_names():
    globals().update(vim_wrapper_factory(name))
