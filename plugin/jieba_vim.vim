" Copyright 2024-2026 Kaiwen Wu. All Rights Reserved.
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


""
" @section Introduction, intro
" @stylized jieba.vim
" @library
" @order intro config commands mappings opt-dependency
" jieba.vim 是一个基于 jieba 中文分词插件.


if exists("g:loaded_jieba_vim")
    finish
endif
let g:loaded_jieba_vim = 1
""
" (默认 1)：是/否 (1/0) 延迟加载 jieba 词典直到有中文出现。
let g:jieba_vim_lazy = get(g:, 'jieba_vim_lazy', 1)

""
" (默认空)：若为非空字符串，加载此文件路径所指向的用户自定义词典。
let g:jieba_vim_user_dict = get(g:, 'jieba_vim_user_dict', '')


""
" (默认 0)：是/否 (1/0) 自动开启 keymap（不包含预览）。
let g:jieba_vim_keymap = get(g:, 'jieba_vim_keymap', 0)

if !has('python3')
    echoerr "python3 is required by jieba.vim"
    finish
endif

py3 import jieba_vim.navigation

function! s:InitWordMotion()
    let l:args = [g:jieba_vim_user_dict, &iskeyword, str2nr(g:jieba_vim_lazy)]
    let l:init_word_motion_err = py3eval(
        \ "jieba_vim.navigation.init_word_motion(*vim.eval('l:args'))")
    if has("nvim")
        " Nvim does not seem to define v:none.
        if l:init_word_motion_err !=# "" && l:init_word_motion_err !=# v:null
            echoerr l:init_word_motion_err
        endif
    else
        if l:init_word_motion_err !=# "" && l:init_word_motion_err !=# v:none
            echoerr l:init_word_motion_err
        endif
    endif
endfunction

call s:InitWordMotion()

""
" 取消按词跳转位置预览
command! JiebaPreviewCancel call <SID>JiebaPreviewCancel()

let s:motions = ["w", "W", "e", "E", "b", "B", "ge", "gE"]

""
" @section Mappings, mappings
" 提供以下 `<Plug>()` 映射，其中 X 表示 Vim word motion 按键，即
" b、B、ge、gE、w、W、e、E：
"
"   - `<Plug>(Jieba_preview_cancel)`：即 |:JiebaPreviewCancel| 命令
"   - `<Plug>(Jieba_preview_X)`：预览增强了的 X 的跳转位置
"   - `<Plug>(Jieba_X)`: 增强了的 X，同时在 normal、operator-pending、visual 三种模式下可用，以及可与 count 协同使用。例如假设 w 被映射到 `<Plug>(Jieba_w)`，那么 3w 将是向后跳三个词，d3w 是删除后三个词
"
"
" 用户可自行在 .vimrc 中将按键映射到这些 `<Plug>()` 映射。例如：
" >
"   nmap <LocalLeader>jw <Plug>(Jieba_preview_w)
"   " 等等，以及
"   map w <Plug>(Jieba_w)
"   " 等等
" <
" 提供快捷开关 g:jieba_vim_keymap，可通过在 .vimrc 中将其设为 1 来开启对八个
" word motion 的 nmap, xmap 和 omap。

""
" @section Optional Dependency, opt-dependency
" 如果用户安装了 `tpope/vim-repeat` (https://github.com/tpope/vim-repeat)，可使用 |.|
" 重复上一次 word operation。例如 `dw.` 相当于 `dwdw`。

function s:JiebaPreviewCancel()
    execute "hi clear JiebaPreview"
endfunction

function! s:JiebaPreview(motion)
    let l:limit = get(g:, "jieba_vim_preview_limits", 0)
    if l:limit < 0
        let l:limit = 99999
    endif
    " In patch-9.1.0844 Vim introduced py3eval({expr}, [{locals}]) api. But in
    " order to work with Vim before that patch, we have to work with this
    " awkward syntax. The same applies below for all calls to `py3eval()`.
    let l:args = [a:motion, getcurpos(), l:limit]
    let l:cursor_positions = py3eval(
        \ "jieba_vim.navigation.preview_nmap(vim.current.buffer, *vim.eval('l:args'))")
    if empty(l:cursor_positions)
        call s:JiebaPreviewCancel()
    else
        execute "hi link JiebaPreview IncSearch"
        let l:pattern = '%' . l:cursor_positions[0][1] . 'c%' . l:cursor_positions[0][0] . 'l'
        for pos in l:cursor_positions[1:]
            let l:pattern .= '|%' . pos[1] . 'c%' . pos[0] . 'l'
        endfor
        for pos in l:cursor_positions
            execute 'match JiebaPreview /\v' . l:pattern . '/'
        endfor
    endif
endfunction

for ky in s:motions
    execute 'nnoremap <silent> <Plug>(Jieba_preview_' . ky . ') :<C-u>call <SID>JiebaPreview("' . ky . '")<CR>'
endfor
nnoremap <silent> <Plug>(Jieba_preview_cancel) :<C-u>call <SID>JiebaPreviewCancel()<CR>

function! JiebaModelNmap(...)
    return py3eval(
        \ "jieba_vim.navigation.nmap(vim.current.buffer, *vim.eval('a:000'))")
endfunction

function! JiebaModelXmap(...)
    return py3eval(
        \ "jieba_vim.navigation.xmap(vim.current.buffer, *vim.eval('a:000'))")
endfunction

function! JiebaModelOmap(...)
    return py3eval(
        \ "jieba_vim.navigation.omap(vim.current.buffer, *vim.eval('a:000'))")
endfunction

function! JiebaNmap(motion, count, model_funcname)
    if a:model_funcname !=# ""
        let l:result_dict = function(a:model_funcname)(a:motion, getcurpos(), a:count)
    else
        let l:result_dict = JiebaModelNmap(a:motion, getcurpos(), a:count)
    endif
    call cursor(l:result_dict["cursor"][1:2])
endfunction

function! JiebaXmap(motion, count, model_funcname)
    execute "normal! \<Esc>"
    let l:orig_mark_a = getpos("'a")
    let l:orig_mark_b = getpos("'b")
    normal! gvomaomb
    let l:visual_begin = getpos("'a")
    let l:visial_end = getpos("'b")
    call setpos("'a", l:orig_mark_a)
    call setpos("'b", l:orig_mark_b)
    if a:model_funcname !=# ""
        let l:result_dict = function(a:model_funcname)(visualmode(), a:motion, l:visual_begin, l:visial_end, a:count)
    else
        let l:result_dict = JiebaModelXmap(visualmode(), a:motion, l:visual_begin, l:visial_end, a:count)
    endif
    execute "normal! " . l:result_dict["visualmode"] . "\<Esc>"
    call setpos("'<", l:result_dict["langle"])
    call setpos("'>", l:result_dict["rangle"])
    normal! gv
endfunction

function! JiebaOmap(motion, repeat, count, operator, register, model_funcname)
    execute "normal! \<Esc>"
    let l:orig_curpos = getcurpos()
    if a:model_funcname !=# ""
        let l:result_dict = function(a:model_funcname)(a:motion, l:orig_curpos, a:count, a:operator)
    else
        let l:result_dict = JiebaModelOmap(a:motion, l:orig_curpos, a:count, a:operator)
    endif
    call cursor(l:result_dict["langle"][1:2])
    " This no-op line effectively sets an undoable checkpoint such that |u|
    " undos all operations up to this line.
    call setline(".", getline("."))
    if l:result_dict["prevent_change"]
        " Land the cursor to potentially a new position.
        call cursor(l:result_dict["cursor"][1:2])
    else
        " We need to use '< and '> marks in this function. Thus the clutters
        " here.

        " Save original states.
        let l:orig_visualmode = visualmode()
        let l:orig_langle = getpos("'<")
        let l:orig_rangle = getpos("'>")
        if l:orig_langle[1] == 0 || l:orig_rangle[1] == 0
            " If either lnum is 0, then |gv| will fail and the visual selection
            " has been lost forever.
            let l:orig_visual_begin = [0, 0, 0, 0]
            let l:orig_visual_end = [0, 0, 0, 0]
        else
            " Otherwise, we need to remember last visual selection in marks 'a
            " and 'b. First, save the original mark positions.
            let l:orig_mark_a = getpos("'a")
            let l:orig_mark_b = getpos("'b")
            " Remember the visual selection range.
            normal! gvomaomb
            let l:orig_visual_begin = getpos("'a")
            let l:orig_visual_end = getpos("'b")
            " Restore mark positions of 'a and 'b.
            call setpos("'a", l:orig_mark_a)
            call setpos("'b", l:orig_mark_b)
            " |gv| will restore last visualmode even if it has been erased
            " deliberately. Thus we need to ensure it stays erased.
            if l:orig_visualmode ==# ""
                call visualmode(1)
            endif
        endif
        let l:orig_selection = &selection

        " Select and execute.
        execute "normal! " . l:result_dict["visualmode"] . "\<Esc>"
        call setpos("'<", l:result_dict["langle"])
        call setpos("'>", l:result_dict["rangle"])
        let &selection = l:result_dict["selection"]
        if a:operator ==# "c" && a:repeat
            execute 'normal! gv"' . a:register . a:operator . @. . "\<Esc>"
        else
            execute 'normal! gv"' . a:register . a:operator . "\<Esc>"
        endif

        " Restore original states.
        let &selection = l:orig_selection
        if l:orig_visualmode ==# ""
            call visualmode(1)
        else
            execute "normal! " . l:orig_visualmode . "\<Esc>"
        endif
        if l:orig_langle[1] == 0 || l:orig_rangle[1] == 0
            call setpos("'<", l:orig_langle)
            call setpos("'>", l:orig_rangle)
        else
            call setpos("'<", l:orig_visual_begin)
            call setpos("'>", l:orig_visual_end)
        endif

        " Land the cursor to potentially a new position.
        call cursor(l:result_dict["cursor"][1:2])

        " Special treatment of d-special in nvim.
        if has("nvim") && a:operator ==# "d" && l:result_dict["visualmode"] ==# "V"
            call cursor(0, virtcol2col(0, line("."), l:orig_curpos[4]))
        endif

        " Special treatment to |c| which needs to drop the user in insert mode.
        if a:operator ==# "c" && a:repeat == 0
            if l:result_dict["cursor"][2] >= col("$")
                startinsert!
            else
                startinsert
            endif
        endif
    endif
endfunction

function! s:JiebaNmap_w()
    call JiebaNmap("w", v:count1, "")
endfunction

function! s:JiebaNmap_W()
    call JiebaNmap("W", v:count1, "")
endfunction

function! s:JiebaNmap_e()
    call JiebaNmap("e", v:count1, "")
endfunction

function! s:JiebaNmap_E()
    call JiebaNmap("E", v:count1, "")
endfunction

function! s:JiebaNmap_b()
    call JiebaNmap("b", v:count1, "")
endfunction

function! s:JiebaNmap_B()
    call JiebaNmap("B", v:count1, "")
endfunction

function! s:JiebaNmap_ge()
    call JiebaNmap("ge", v:count1, "")
endfunction

function! s:JiebaNmap_gE()
    call JiebaNmap("gE", v:count1, "")
endfunction

nnoremap <silent> <Plug>(Jieba_w) :<C-u>call <SID>JiebaNmap_w()<CR>
nnoremap <silent> <Plug>(Jieba_W) :<C-u>call <SID>JiebaNmap_W()<CR>
nnoremap <silent> <Plug>(Jieba_e) :<C-u>call <SID>JiebaNmap_e()<CR>
nnoremap <silent> <Plug>(Jieba_E) :<C-u>call <SID>JiebaNmap_E()<CR>
nnoremap <silent> <Plug>(Jieba_b) :<C-u>call <SID>JiebaNmap_b()<CR>
nnoremap <silent> <Plug>(Jieba_B) :<C-u>call <SID>JiebaNmap_B()<CR>
nnoremap <silent> <Plug>(Jieba_ge) :<C-u>call <SID>JiebaNmap_ge()<CR>
nnoremap <silent> <Plug>(Jieba_gE) :<C-u>call <SID>JiebaNmap_gE()<CR>

function! s:JiebaXmap_w()
    call JiebaXmap("w", v:count1, "")
endfunction

function! s:JiebaXmap_W()
    call JiebaXmap("W", v:count1, "")
endfunction

function! s:JiebaXmap_e()
    call JiebaXmap("e", v:count1, "")
endfunction

function! s:JiebaXmap_E()
    call JiebaXmap("E", v:count1, "")
endfunction

function! s:JiebaXmap_b()
    call JiebaXmap("b", v:count1, "")
endfunction

function! s:JiebaXmap_B()
    call JiebaXmap("B", v:count1, "")
endfunction

function! s:JiebaXmap_ge()
    call JiebaXmap("ge", v:count1, "")
endfunction

function! s:JiebaXmap_gE()
    call JiebaXmap("gE", v:count1, "")
endfunction

xnoremap <silent> <Plug>(Jieba_w) <Esc>:<C-u>call <SID>JiebaXmap_w()<CR>
xnoremap <silent> <Plug>(Jieba_W) <Esc>:<C-u>call <SID>JiebaXmap_W()<CR>
xnoremap <silent> <Plug>(Jieba_e) <Esc>:<C-u>call <SID>JiebaXmap_e()<CR>
xnoremap <silent> <Plug>(Jieba_E) <Esc>:<C-u>call <SID>JiebaXmap_E()<CR>
xnoremap <silent> <Plug>(Jieba_b) <Esc>:<C-u>call <SID>JiebaXmap_b()<CR>
xnoremap <silent> <Plug>(Jieba_B) <Esc>:<C-u>call <SID>JiebaXmap_B()<CR>
xnoremap <silent> <Plug>(Jieba_ge) <Esc>:<C-u>call <SID>JiebaXmap_ge()<CR>
xnoremap <silent> <Plug>(Jieba_gE) <Esc>:<C-u>call <SID>JiebaXmap_gE()<CR>

function s:JiebaOmap_internal_w()
    silent! call repeat#setreg("\<Plug>(Jieba_internal_o_w)", v:register)
    call JiebaOmap("w", 1, v:count1, v:operator, v:register, "")
    silent! call repeat#set("\<Plug>(Jieba_internal_o_w)", v:count1)
endfunction

function! s:JiebaOmap_w()
    silent! call repeat#setreg("\<Plug>(Jieba_internal_o_w)", v:register)
    call JiebaOmap("w", 0, v:count1, v:operator, v:register, "")
    silent! call repeat#set("\<Plug>(Jieba_internal_o_w)", v:count1)
endfunction

function! s:JiebaOmap_internal_W()
    silent! call repeat#setreg("\<Plug>(Jieba_internal_o_W)", v:register)
    call JiebaOmap("W", 1, v:count1, v:operator, v:register, "")
    silent! call repeat#set("\<Plug>(Jieba_internal_o_W)", v:count1)
endfunction

function! s:JiebaOmap_W()
    silent! call repeat#setreg("\<Plug>(Jieba_internal_o_W)", v:register)
    call JiebaOmap("W", 0, v:count1, v:operator, v:register, "")
    silent! call repeat#set("\<Plug>(Jieba_internal_o_W)", v:count1)
endfunction

function! s:JiebaOmap_internal_e()
    silent! call repeat#setreg("\<Plug>(Jieba_internal_o_e)", v:register)
    call JiebaOmap("e", 1, v:count1, v:operator, v:register, "")
    silent! call repeat#set("\<Plug>(Jieba_internal_o_e)", v:count1)
endfunction

function! s:JiebaOmap_e()
    silent! call repeat#setreg("\<Plug>(Jieba_internal_o_e)", v:register)
    call JiebaOmap("e", 0, v:count1, v:operator, v:register, "")
    silent! call repeat#set("\<Plug>(Jieba_internal_o_e)", v:count1)
endfunction

function! s:JiebaOmap_internal_E()
    silent! call repeat#setreg("\<Plug>(Jieba_internal_o_E)", v:register)
    call JiebaOmap("E", 1, v:count1, v:operator, v:register, "")
    silent! call repeat#set("\<Plug>(Jieba_internal_o_E)", v:count1)
endfunction

function! s:JiebaOmap_E()
    silent! call repeat#setreg("\<Plug>(Jieba_internal_o_E)", v:register)
    call JiebaOmap("E", 0, v:count1, v:operator, v:register, "")
    silent! call repeat#set("\<Plug>(Jieba_internal_o_E)", v:count1)
endfunction

function! s:JiebaOmap_internal_b()
    silent! call repeat#setreg("\<Plug>(Jieba_internal_o_b)", v:register)
    call JiebaOmap("b", 1, v:count1, v:operator, v:register, "")
    silent! call repeat#set("\<Plug>(Jieba_internal_o_b)", v:count1)
endfunction

function! s:JiebaOmap_b()
    silent! call repeat#setreg("\<Plug>(Jieba_internal_o_b)", v:register)
    call JiebaOmap("b", 0, v:count1, v:operator, v:register, "")
    silent! call repeat#set("\<Plug>(Jieba_internal_o_b)", v:count1)
endfunction

function! s:JiebaOmap_internal_B()
    silent! call repeat#setreg("\<Plug>(Jieba_internal_o_B)", v:register)
    call JiebaOmap("B", 1, v:count1, v:operator, v:register, "")
    silent! call repeat#set("\<Plug>(Jieba_internal_o_B)", v:count1)
endfunction

function! s:JiebaOmap_B()
    silent! call repeat#setreg("\<Plug>(Jieba_internal_o_B)", v:register)
    call JiebaOmap("B", 0, v:count1, v:operator, v:register, "")
    silent! call repeat#set("\<Plug>(Jieba_internal_o_B)", v:count1)
endfunction

function! s:JiebaOmap_internal_ge()
    silent! call repeat#setreg("\<Plug>(Jieba_internal_o_ge)", v:register)
    call JiebaOmap("ge", 1, v:count1, v:operator, v:register, "")
    silent! call repeat#set("\<Plug>(Jieba_internal_o_ge)", v:count1)
endfunction

function! s:JiebaOmap_ge()
    silent! call repeat#setreg("\<Plug>(Jieba_internal_o_ge)", v:register)
    call JiebaOmap("ge", 0, v:count1, v:operator, v:register, "")
    silent! call repeat#set("\<Plug>(Jieba_internal_o_ge)", v:count1)
endfunction

function! s:JiebaOmap_internal_gE()
    silent! call repeat#setreg("\<Plug>(Jieba_internal_o_gE)", v:register)
    call JiebaOmap("gE", 1, v:count1, v:operator, v:register, "")
    silent! call repeat#set("\<Plug>(Jieba_internal_o_gE)", v:count1)
endfunction

function! s:JiebaOmap_gE()
    silent! call repeat#setreg("\<Plug>(Jieba_internal_o_gE)", v:register)
    call JiebaOmap("gE", 0, v:count1, v:operator, v:register, "")
    silent! call repeat#set("\<Plug>(Jieba_internal_o_gE)", v:count1)
endfunction

nnoremap <silent> <Plug>(Jieba_internal_o_w) :<C-u>call <SID>JiebaOmap_internal_w()<CR>
nnoremap <silent> <Plug>(Jieba_internal_o_W) :<C-u>call <SID>JiebaOmap_internal_W()<CR>
nnoremap <silent> <Plug>(Jieba_internal_o_e) :<C-u>call <SID>JiebaOmap_internal_e()<CR>
nnoremap <silent> <Plug>(Jieba_internal_o_E) :<C-u>call <SID>JiebaOmap_internal_E()<CR>
nnoremap <silent> <Plug>(Jieba_internal_o_b) :<C-u>call <SID>JiebaOmap_internal_b()<CR>
nnoremap <silent> <Plug>(Jieba_internal_o_B) :<C-u>call <SID>JiebaOmap_internal_B()<CR>
nnoremap <silent> <Plug>(Jieba_internal_o_ge) :<C-u>call <SID>JiebaOmap_internal_ge()<CR>
nnoremap <silent> <Plug>(Jieba_internal_o_gE) :<C-u>call <SID>JiebaOmap_internal_gE()<CR>

onoremap <silent> <Plug>(Jieba_w) <Esc>:<C-u>call <SID>JiebaOmap_w()<CR>
onoremap <silent> <Plug>(Jieba_W) <Esc>:<C-u>call <SID>JiebaOmap_W()<CR>
onoremap <silent> <Plug>(Jieba_e) <Esc>:<C-u>call <SID>JiebaOmap_e()<CR>
onoremap <silent> <Plug>(Jieba_E) <Esc>:<C-u>call <SID>JiebaOmap_E()<CR>
onoremap <silent> <Plug>(Jieba_b) <Esc>:<C-u>call <SID>JiebaOmap_b()<CR>
onoremap <silent> <Plug>(Jieba_B) <Esc>:<C-u>call <SID>JiebaOmap_B()<CR>
onoremap <silent> <Plug>(Jieba_ge) <Esc>:<C-u>call <SID>JiebaOmap_ge()<CR>
onoremap <silent> <Plug>(Jieba_gE) <Esc>:<C-u>call <SID>JiebaOmap_gE()<CR>

let s:modes = ["n", "x", "o"]
if g:jieba_vim_keymap
    for ky in s:motions
        for md in s:modes
            execute md . "map " . ky . " <Plug>(Jieba_" . ky . ")"
        endfor
    endfor
endif

augroup jieba_vim_update_isk
    autocmd!
    autocmd OptionSet iskeyword call py3eval("jieba_vim.navigation.update_isk(vim.eval('&iskeyword'))")
augroup END
