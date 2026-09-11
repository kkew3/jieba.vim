" Copyright 2026 Kaiwen Wu. All Rights Reserved.
"
" Licensed under the Apache License, Version 2.0 (the "License"); you may not
" use this file except in compliance with the License. You may obtain a copy
" of the License at
"
"     http://www.apache.org/licenses/LICENSE-2.0
"
" Unless required by applicable law or agreed to in writing, software
" distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
" WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
" License for the specific language governing permissions and limitations
" under the License.


function! jieba_vim#utils#del_to_cursor(start_col, cur_col)
    let l:line = getline(".")
    if a:start_col > 1
        let l:head = l:line[0:a:start_col - 2]
    else
        let l:head = ""
    endif
    if a:cur_col < col("$")
        let l:tail = l:line[a:cur_col - 1:]
    else
        let l:tail = ""
    endif
    let l:line_modified = l:head . l:tail
    call setline(".", l:line_modified)
    call cursor(0, a:start_col)
endfunction

function! jieba_vim#utils#consume_chars()
    if exists("$JIEBA_TEST_CASE")
        return
    endif
    while 1
        let l:ch = getchar(1)
        " Testing against 27 (\<Esc>) is necessary; otherwise Vim will crash.
        if l:ch ==# 0 || l:ch ==# 27
            break
        endif
        call getchar(0)
    endwhile
endfunction

function! jieba_vim#utils#is_forward_motion(motion)
    return a:motion ==? "w" || a:motion ==? "e" || a:motion ==? "iw" || a:motion ==? "aw"
endfunction

function jieba_vim#utils#update_isk()
    if has("nvim")
        lua jieba_vim:update_isk(vim.o.iskeyword)
    else
        py3 jieba_vim.navigation.update_isk(vim.eval('&iskeyword'))
    endif
endfunction
