# Copyright 2024-2026 Kaiwen Wu. All Rights Reserved.
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

from . import jieba_vim_rs

word_motion = None


def as_bytes(s):
    if isinstance(s, str):
        return s.encode("utf-8")
    return s


def ints(arr):
    return [int(x) for x in arr]


def init_word_motion(user_dict, isk, lazy):
    """Return error message. Empty error message means no error."""
    if not user_dict:
        user_dict = None
    isk = as_bytes(isk)
    lazy = bool(int(lazy))

    global word_motion
    if word_motion is not None:
        return ""
    try:
        if lazy:
            word_motion = jieba_vim_rs.LazyWordMotion(isk, user_dict)
        else:
            word_motion = jieba_vim_rs.WordMotion(isk, user_dict)
    except (IOError, ValueError):
        return f"jieba.vim: failed to load user dict: {user_dict}"


def nmap(buffer, motion, cursor, count):
    # We have to do these type conversion because `vim.eval("a:000")` syntax
    # in jieba_vim.vim converts all values to str. For example, `cursor` should
    # be a list of 4 or 5 ints, but we will receive a list of strings.
    motion = as_bytes(motion)
    cursor = ints(cursor)
    count = int(count)
    return word_motion.nmap(buffer, motion, cursor, count)


def xmap(buffer, visualmode, motion, visual_begin, visual_end, count):
    visualmode = as_bytes(visualmode)
    motion = as_bytes(motion)
    visual_begin = ints(visual_begin)
    visual_end = ints(visual_end)
    count = int(count)
    return word_motion.xmap(
        buffer, visualmode, motion, visual_begin, visual_end, count
    )


def omap(buffer, motion, cursor, count, operator):
    motion = as_bytes(motion)
    cursor = ints(cursor)
    count = int(count)
    operator = as_bytes(operator)
    return word_motion.omap(buffer, motion, cursor, count, operator)


def imap_ctrl_w(buffer, cursor):
    cursor = ints(cursor)
    return word_motion.imap_ctrl_w(buffer, cursor)


def preview_nmap(buffer, motion, cursor, preview_limit):
    motion = as_bytes(motion)
    cursor = ints(cursor)
    preview_limit = int(preview_limit)
    return word_motion.preview_nmap(buffer, motion, cursor, preview_limit)


def update_isk(isk):
    isk = as_bytes(isk)
    word_motion.set_isk(isk)
