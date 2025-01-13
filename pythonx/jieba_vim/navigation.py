# Copyright 2024 Kaiwen Wu. All Rights Reserved.
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
"""
These names are dynamically defined in this module::

    - nmap_w
    - nmap_W
    - xmap_w
    - xmap_W
    - omap_w
    - omap_W
    - nmap_e
    - nmap_E
    - xmap_e
    - xmap_E
    - omap_e
    - omap_E
    - nmap_b
    - nmap_B
    - xmap_b
    - xmap_B
    - omap_b
    - omap_B
    - nmap_ge
    - nmap_gE
    - xmap_ge
    - xmap_gE
    - omap_ge
    - omap_gE
"""
import vim

from . import jieba_vim_rs

word_motion = None


def upperbound_count(count):
    """
    Upperbound the count at 2**64-1. This assumes the use of u64 type for
    count.
    """
    return min(18446744073709551615, count)


def get_register_value(register):
    # The register is quoted by single quotes because no register is named `'`.
    return vim.eval("getreg('{}')".format(register))


def set_register_value(register, value):
    # First escape backslashes and double quotes.
    value = value.replace('\\', '\\\\').replace('"', '\\"')
    vim.command("call setreg('{}', \"{}\")".format(register, value))


def _init_word_motion():
    global word_motion
    if word_motion is not None:
        return
    user_dict = vim.eval('g:jieba_vim_user_dict') or None
    try:
        if int(vim.eval('g:jieba_vim_lazy')):
            word_motion = jieba_vim_rs.LazyWordMotion(user_dict)
        else:
            word_motion = jieba_vim_rs.WordMotion(user_dict)
    except (IOError, ValueError):
        vim.command('echoerr "jieba.vim: failed to load user dict: {}"'.format(
            user_dict))


_init_word_motion()


def _vim_wrapper_factory_n(motion_name):
    fun_name = 'nmap_' + motion_name

    def _motion_wrapper(count):
        count = upperbound_count(count)
        method = getattr(word_motion, fun_name)
        output = method(vim.current.buffer, vim.current.window.cursor, count)
        vim.current.window.cursor = output.cursor

    return {fun_name: _motion_wrapper}


def _vim_wrapper_factory_x(motion_name):
    fun_name = 'xmap_' + motion_name

    def _motion_wrapper(count):
        count = upperbound_count(count)
        method = getattr(word_motion, fun_name)
        virtualedit_config = vim.eval('&virtualedit')
        vim.command('set virtualedit=onemore')
        # Handle the case where cursor is one character after the last
        # character of the buffer in visual mode.
        line = vim.current.window.cursor[0]
        col_gt = int(vim.eval('''col("'>")''')) - 1
        if col_gt >= len(vim.current.buffer[line - 1].encode('utf-8')):
            output = method(vim.current.buffer, (line, col_gt), count)
        else:
            output = method(vim.current.buffer, vim.current.window.cursor,
                            count)
        vim.current.window.cursor = output.cursor
        # The `m>gv` trick reference:
        # https://github.com/svermeulen/vim-NotableFt/blob/01732102c1d8c7b7bd6e221329e37685aa4ab41a/plugin/NotableFt.vim#L32
        vim.command('normal! m>gv')
        vim.command('set virtualedit={}'.format(virtualedit_config))

    return {fun_name: _motion_wrapper}


def _vim_wrapper_factory_omap_w(motion_name):
    assert motion_name in ['w', 'W']
    fun_name = 'omap_' + motion_name

    def _motion_wrapper(register, operator, count):
        vim.command(
            'silent! call repeat#setreg("\\<Plug>(Jieba_internal_o_{})", \'{}\')'
            .format(motion_name, register))
        count = upperbound_count(count)
        method = getattr(word_motion, fun_name)
        virtualedit_config = vim.eval('&virtualedit')
        output = method(vim.current.buffer, vim.current.window.cursor,
                        operator, count)
        col_before = vim.current.window.cursor[1]
        vim.command('set virtualedit=onemore')
        # `output.cursor[1] + 1` because vim column starts from 1 whereas vim
        # python api column starts from 0.
        vim.command(
            '''execute 'silent normal! "{}{}:call cursor({}, {})' . "\\<CR>"'''
            .format(register, operator, output.cursor[0],
                    output.cursor[1] + 1))
        if operator == 'c':
            # Running `c` in `normal!` as above will shift the cursor one more
            # character to the left; so we need to shift back one character.
            if col_before > 0:
                vim.command('normal! l')
            vim.command('startinsert')
        vim.command('set virtualedit={}'.format(virtualedit_config))
        vim.command(
            'silent! call repeat#set("\\<Plug>(Jieba_internal_o_{})", {})'
            .format(motion_name, count))

    return {fun_name: _motion_wrapper}


def _vim_wrapper_factory_omap_e(motion_name):
    assert motion_name in ['e', 'E']
    fun_name = 'omap_' + motion_name

    def _motion_wrapper(register, operator, count):
        vim.command(
            'silent! call repeat#setreg("\\<Plug>(Jieba_internal_o_{})", \'{}\')'
            .format(motion_name, register))
        count = upperbound_count(count)
        method = getattr(word_motion, fun_name)
        virtualedit_config = vim.eval('&virtualedit')
        output = method(vim.current.buffer, vim.current.window.cursor,
                        operator, count)
        line_before = vim.current.window.cursor[0]
        col_before = vim.current.window.cursor[1]
        # This will be used in d-special case below.
        chars_before_cursor = vim.current.buffer[line_before - 1][:col_before]
        vim.command('set virtualedit=onemore')
        # `output.cursor[1] + 1` because vim column starts from 1 whereas vim
        # python api column starts from 0.
        vim.command(
            '''execute 'silent normal! "{}{}v:call cursor({}, {})' . "\\<CR>"'''
            .format(register, operator, output.cursor[0],
                    output.cursor[1] + 1))
        reg_value = get_register_value(register)
        if operator == 'c':
            # Running `c` in `normal!` as above will shift the cursor one more
            # character to the left; so we need to shift back one character.
            if col_before:
                vim.command('normal! l')
            vim.command('startinsert')
        elif operator == 'd' and output.d_special:
            vim.command('normal! "{}dd'.format(register))
            # `reg_value2` consists of `chars_before_cursor` and the rest. We
            # need this order: `chars_before_cursor`, `reg_value`, the rest.
            reg_value2 = get_register_value(register)
            reg_value = (
                chars_before_cursor + reg_value + reg_value2[col_before:])
            set_register_value(register, reg_value)
            # We don't need this block because somehow having
            # virtualedit=onemore overcomes the cursor position issue.
            #if int(vim.eval('has("nvim")')):
            #    vim.command("""execute 'silent call cursor(line("."), {})'"""
            #                .format(col_before + 1))
        vim.command('set virtualedit={}'.format(virtualedit_config))
        vim.command(
            'silent! call repeat#set("\\<Plug>(Jieba_internal_o_{})", {})'
            .format(motion_name, count))

    return {fun_name: _motion_wrapper}


def _vim_wrapper_factory_omap_b(motion_name):
    assert motion_name in ['b', 'B']
    fun_name = 'omap_' + motion_name

    def _motion_wrapper(register, operator, count):
        vim.command(
            'silent! call repeat#setreg("\\<Plug>(Jieba_internal_o_{})", \'{}\')'
            .format(motion_name, register))
        count = upperbound_count(count)
        method = getattr(word_motion, fun_name)
        output = method(vim.current.buffer, vim.current.window.cursor, count)
        if output.prevent_change:
            vim.current.window.cursor = output.cursor
        else:
            # `output.cursor[1] + 1` because vim column starts from 1 whereas
            # vim python api column starts from 0.
            vim.command(
                '''execute 'silent normal! "{}{}:call cursor({}, {})' . "\\<CR>"'''
                .format(register, operator, output.cursor[0],
                        output.cursor[1] + 1))
            if operator == 'c':
                # Running `c` in `normal!` as above will shift the cursor one
                # more character to the left; so we need to shift back one
                # character.
                if output.cursor[1] > 0:
                    vim.command('normal! l')
                vim.command('startinsert')
        vim.command(
            'silent! call repeat#set("\\<Plug>(Jieba_internal_o_{})", {})'
            .format(motion_name, count))

    return {fun_name: _motion_wrapper}


def _vim_wrapper_factory_omap_ge(motion_name):
    assert motion_name in ['ge', 'gE']
    fun_name = 'omap_' + motion_name

    def _motion_wrapper(register, operator, count):
        vim.command(
            'silent! call repeat#setreg("\\<Plug>(Jieba_internal_o_{})", \'{}\')'
            .format(motion_name, register))
        count = upperbound_count(count)
        method = getattr(word_motion, fun_name)
        output = method(vim.current.buffer, vim.current.window.cursor,
                        operator, count)
        col_before = vim.current.window.cursor[1]
        if output.prevent_change:
            vim.current.window.cursor = output.cursor
        else:
            # `output.cursor[1] + 1` because vim column starts from 1 whereas
            # vim python api column starts from 0.
            vim.command(
                '''execute 'silent normal! "{}{}v:call cursor({}, {})' . "\\<CR>"'''
                .format(register, operator, output.cursor[0],
                        output.cursor[1] + 1))
            reg_value = get_register_value(register)
            if operator == 'c':
                # Running `c` in `normal!` as above will shift the cursor one
                # more character to the left; so we need to shift back one
                # character.
                if output.cursor[1] > 0:
                    vim.command('normal! l')
                vim.command('startinsert')
            elif operator == 'd' and output.d_special:
                vim.command('normal! "{}dd'.format(register))
                reg_value += get_register_value(register)
                set_register_value(register, reg_value)
                if int(vim.eval('has("nvim")')):
                    vim.command(
                        '''execute "silent call cursor(line('.'), {})"'''
                        .format(col_before + 1))
        vim.command(
            'silent! call repeat#set("\\<Plug>(Jieba_internal_o_{})", {})'
            .format(motion_name, count))

    return {fun_name: _motion_wrapper}


def _define_functions():
    for mo in ['w', 'W', 'e', 'E', 'b', 'B', 'ge', 'gE']:
        globals().update(_vim_wrapper_factory_n(mo))
        globals().update(_vim_wrapper_factory_x(mo))
        if mo in ['e', 'E']:
            globals().update(_vim_wrapper_factory_omap_e(mo))
        elif mo in ['b', 'B']:
            globals().update(_vim_wrapper_factory_omap_b(mo))
        elif mo in ['ge', 'gE']:
            globals().update(_vim_wrapper_factory_omap_ge(mo))
        else:
            globals().update(_vim_wrapper_factory_omap_w(mo))


_define_functions()
