import vim

from .pysrc import navigation
from .init import jieba_initialized


@jieba_initialized
def backward_start():
    vim.current.window.cursor = navigation.backward_start(
        vim.current.buffer, vim.current.window.cursor)
