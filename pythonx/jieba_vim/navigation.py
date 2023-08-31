import vim

from .pysrc import navigation
from .init import jieba_initialized


def responding_to_vcount(func):
    def _wrapper():
        for _ in range(int(vim.eval('v:count1'))):
            func()

    return _wrapper


def in_visual_mode(func):
    def _wrapper():
        # Reference: https://stackoverflow.com/q/16212801/7881370
        vim.command('normal! gv')
        func()

    return _wrapper


@jieba_initialized
@responding_to_vcount
def backward_start():
    vim.current.window.cursor = navigation.backward_start(
        vim.current.buffer, vim.current.window.cursor)


# known issue: responding_to_vcount won't work in visual mode
backward_start_v = in_visual_mode(backward_start)
