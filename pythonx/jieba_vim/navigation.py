"""
These names are dynamically defined in this module::

    - wordmotion_b
    - wordmotion_B
    - wordmotion_ge
    - wordmotion_gE
    - wordmotion_w
    - wordmotion_W
    - wordmotion_e
    - wordmotion_E
"""
import vim

from . import jieba_navi_rs as navigation


def vim_wrapper_factory(py_navi_func_name):
    py_navi_func = getattr(navigation, py_navi_func_name)

    def _wrapper(count):
        for _ in range(count):
            vim.current.window.cursor = py_navi_func(vim.current.buffer,
                                                     vim.current.window.cursor)

    return {py_navi_func_name: _wrapper}


def get_navi_func_names():
    motions = ['b', 'B', 'ge', 'gE', 'w', 'W', 'e', 'E']
    return ['wordmotion_' + m for m in motions]


for name in get_navi_func_names():
    globals().update(vim_wrapper_factory(name))
