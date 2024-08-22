"""
These names are dynamically defined in this module::

    - wordmotion_b
    - wordmotion_b_v
    - wordmotion_B
    - wordmotion_B_v
    - wordmotion_ge
    - wordmotion_ge_v
    - wordmotion_gE
    - wordmotion_gE_v
    - wordmotion_w
    - wordmotion_w_v
    - wordmotion_W
    - wordmotion_W_v
    - wordmotion_e
    - wordmotion_e_v
    - wordmotion_E
    - wordmotion_E_v
"""
import vim

from . import jieba_navi_rs as navigation


def vim_wrapper_factory(py_navi_func_name):
    py_navi_func = getattr(navigation, py_navi_func_name)

    def _wrapper():
        for _ in range(int(vim.eval('v:count1'))):
            vim.current.window.cursor = py_navi_func(vim.current.buffer,
                                                     vim.current.window.cursor)

    # known issue: responding_to_vcount won't work in visual mode
    def _wrapper_v():
        # Reference: https://stackoverflow.com/q/16212801/7881370
        vim.command('normal! gv')
        vim.current.window.cursor = py_navi_func(vim.current.buffer,
                                                 vim.current.window.cursor)

    return {py_navi_func_name: _wrapper, py_navi_func_name + '_v': _wrapper_v}


def get_navi_func_names():
    motions = ['b', 'B', 'ge', 'gE', 'w', 'W', 'e', 'E']
    return ['wordmotion_' + m for m in motions]


for name in get_navi_func_names():
    globals().update(vim_wrapper_factory(name))
