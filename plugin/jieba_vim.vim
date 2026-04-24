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

if !has("nvim") && !has('python3')
    echoerr "python3 is required by jieba.vim"
    finish
endif

" Reference: https://github.com/junegunn/fzf/blob/master/plugin/fzf.vim
let s:is_win = has("win32") || has("win64")
if s:is_win && &shellslash
    set noshellslash
    let s:base_dir = expand("<sfile>:h:h")
    set shellslash
else
    let s:base_dir = expand("<sfile>:h:h")
endif

if s:is_win && !has("win32unix") && has("nvim")
    let s:cdylib_suffix = ".dll"
elseif s:is_win && !has("win32unix")
    let s:cdylib_suffix = ".pyd"
else
    let s:cdylib_suffix = ".so"
endif

function! s:CheckCdylib()
    if has("nvim")
        if filereadable(s:base_dir . "/lua/jieba_vim/jieba_vim_rs" . s:cdylib_suffix)
            lua jieba_vim = require("jieba_vim")
            let s:loaded_jieba_vim_cdylib = 1
        else
            let s:loaded_jieba_vim_cdylib = 0
        endif
    else
        if filereadable(s:base_dir . "/pythonx/jieba_vim/jieba_vim_rs" . s:cdylib_suffix)
            py3 import jieba_vim.navigation
            let s:loaded_jieba_vim_cdylib = 1
        else
            let s:loaded_jieba_vim_cdylib = 0
        endif
    endif
endfunction
call s:CheckCdylib()

function! s:InitWordMotion() abort
    if !s:loaded_jieba_vim_cdylib
        return
    endif
    let l:args = [g:jieba_vim_user_dict, &iskeyword, str2nr(g:jieba_vim_lazy)]
    if has("nvim")
        let l:init_word_motion_err = luaeval("jieba_vim:init_word_motion(unpack(_A))", l:args)
        if l:init_word_motion_err !=# ""
            echoerr l:init_word_motion_err
        endif
    else
        let l:init_word_motion_err = py3eval(
            \ "jieba_vim.navigation.init_word_motion(*vim.eval('l:args'))")
        if l:init_word_motion_err !=# "" && l:init_word_motion_err !=# v:none
            echoerr l:init_word_motion_err
        endif
    endif
    let s:loaded_jieba_vim_word_motion = 1
endfunction

let s:loaded_jieba_vim_word_motion = 0
call s:InitWordMotion()

""
" 取消按词跳转位置预览
command! JiebaPreviewCancel call <SID>JiebaPreviewCancel()

let s:motions = ["w", "W", "e", "E", "b", "B", "ge", "gE"]
let s:objects = ["iw", "iW", "aw", "aW"]

""
" @section Mappings, mappings
" 提供以下 `<Plug>()` 映射，其中 X 表示 Vim word motion/text object 按键，即
" b、B、ge、gE、w、W、e、E、iw、iW、aw、aW：
"
"   - `<Plug>(Jieba_preview_cancel)`：即 |:JiebaPreviewCancel| 命令
"   - `<Plug>(Jieba_preview_X)`：预览增强了的 X 的跳转位置
"   - `<Plug>(Jieba_X)`: 增强了的 X，同时在 normal、operator-pending、visual 三种模式下可用，以及可与 count 协同使用。例如假设 w 被映射到 `<Plug>(Jieba_w)`，那么 3w 将是向后跳三个词，d3w 是删除后三个词
"
" 注意 word text object (即 iw、iW、aw、aW 没有 normal 模式下的映射)。
"
" 用户可自行在 .vimrc 中将按键映射到这些 `<Plug>()` 映射。例如：
" >
"   nmap <LocalLeader>jw <Plug>(Jieba_preview_w)
"   " 等等，以及
"   map w <Plug>(Jieba_w)
"   " 等等
" <
" 提供快捷开关 g:jieba_vim_keymap，可通过在 .vimrc 中将其设为 1 来开启对十二个
" word motion/text object 的 nmap, xmap 和 omap。

""
" @section Optional Dependency, opt-dependency
" 如果用户安装了 `tpope/vim-repeat` (https://github.com/tpope/vim-repeat)，可使用 |.|
" 重复上一次 word operation。例如 `dw.` 相当于 `dwdw`。

function s:JiebaPreviewCancel()
    execute "hi clear JiebaPreview"
endfunction

function s:JiebaModelPreview(...)
    if !s:loaded_jieba_vim_cdylib 
        throw "cdylib unloaded; run jieba_vim#install() first"
    endif
    if !s:loaded_jieba_vim_word_motion
        throw "word_motion uninitialized; check jieba_vim config"
    endif

    if has("nvim")
        return luaeval("jieba_vim:preview_nmap(jieba_vim.buffer, unpack(_A))",
            \ a:000)
    else
        " In patch-9.1.0844 Vim introduced py3eval({expr}, [{locals}]) api.
        " But in order to work with Vim before that patch, we have to work
        " with this awkward syntax. The same applies below for all calls to
        " `py3eval()`.
        let l:args = a:000
        return py3eval(
            \ "jieba_vim.navigation.preview_nmap(vim.current.buffer, *vim.eval('l:args'))")
    endif
endfunction

function! s:JiebaPreview(motion)
    let l:limit = get(g:, "jieba_vim_preview_limits", 0)
    if l:limit < 0
        let l:limit = 99999
    endif
    let l:cursor_positions = s:JiebaModelPreview(a:motion, getcurpos(), l:limit)
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
    if !s:loaded_jieba_vim_cdylib
        throw "cdylib unloaded; run jieba_vim#install() first"
    endif
    if !s:loaded_jieba_vim_word_motion
        throw "word_motion uninitialized; check jieba_vim config"
    endif

    if has("nvim")
        return luaeval("jieba_vim:nmap(jieba_vim.buffer, unpack(_A))", a:000)
    else
        return py3eval(
            \ "jieba_vim.navigation.nmap(vim.current.buffer, *vim.eval('a:000'))")
    endif
endfunction

function! JiebaModelXmap(...)
    if !s:loaded_jieba_vim_cdylib
        throw "cdylib unloaded; run jieba_vim#install() first"
    endif
    if !s:loaded_jieba_vim_word_motion
        throw "word_motion uninitialized; check jieba_vim config"
    endif

    if has("nvim")
        return luaeval("jieba_vim:xmap(jieba_vim.buffer, unpack(_A))", a:000)
    else
        return py3eval(
            \ "jieba_vim.navigation.xmap(vim.current.buffer, *vim.eval('a:000'))")
    endif
endfunction

function! JiebaModelOmap(...)
    if !s:loaded_jieba_vim_cdylib
        throw "cdylib unloaded; run jieba_vim#install() first"
    endif
    if !s:loaded_jieba_vim_word_motion
        throw "word_motion uninitialized; check jieba_vim config"
    endif

    if has("nvim")
        return luaeval("jieba_vim:omap(jieba_vim.buffer, unpack(_A))", a:000)
    else
        return py3eval(
            \ "jieba_vim.navigation.omap(vim.current.buffer, *vim.eval('a:000'))")
    endif
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

function! s:IsForwardMotion(motion)
    return a:motion ==? "w" || a:motion ==? "e" || a:motion ==? "iw" || a:motion ==? "aw"
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

    " Check if we are selecting an empty region.
    if l:result_dict["langle"] ==# l:result_dict["rangle"]
        \ && l:result_dict["selection"] ==# "exclusive"
        \ && l:result_dict["visualmode"] !=# "V"
        \ && !l:result_dict["prevent_change"]
        \ && stridx(&cpoptions, "E") >= 0
        let l:result_dict["prevent_change"] = 1
    endif

    if l:result_dict["prevent_change"]
        " Land the cursor to potentially a new position.
        call cursor(l:result_dict["cursor"][1:2])
    else
        " Save original states.
        let l:orig_mark_a = getpos("'a")
        let l:orig_startofline = &startofline
        let l:orig_eventignore = &eventignore

        " We need this option for cursor to be correctly positioned after
        " d-special.
        set startofline

        " Ignore certain events to match the builtin behavior.
        let l:ignored_events = "InsertEnter,InsertLeave"
        if exists('##ModeChanged')
            let l:ignored_events = l:ignored_events . ",ModeChanged"
        endif
        let &eventignore = l:ignored_events

        " ===
        " Select ...
        if s:IsForwardMotion(a:motion)
            let l:start_pos = l:result_dict["langle"]
            let l:end_pos = l:result_dict["rangle"]
        else
            let l:start_pos = l:result_dict["rangle"]
            let l:end_pos = l:result_dict["langle"]
        endif
        call cursor(l:start_pos[1:2])

        " We need this line of code to decide whether to re-position cursor
        " after d-special when 'startofline' is unset.
        let l:need_repos = !empty(getline(l:end_pos[1]))

        " .. and execute
        let l:cont = a:operator ==# "c" && a:repeat ? @. : ""
        if l:result_dict["visualmode"] ==# "V"
            " Linewise operation.
            let l:op_lines = l:end_pos[1] - l:start_pos[1] + 1
            execute 'normal! "' . a:register . l:op_lines . a:operator . a:operator . l:cont
        else
            " Characterwise operation.
            let l:v = l:result_dict["selection"] ==# "inclusive" ? "v" : ""
            call setpos("'a", l:end_pos)
            execute 'normal! "' . a:register . a:operator . l:v . "`a" . l:cont
        endif
        " ===

        " Restore original states.
        let &eventignore = l:orig_eventignore
        let &startofline = l:orig_startofline
        call setpos("'a", l:orig_mark_a)

        " Land the cursor to potentially a new position.
        " If we have used d-special, the cursor should already be placed by
        " Vim.
        if l:result_dict["visualmode"] !=# "V"
            call cursor(l:result_dict["cursor"][1:2])
        endif

        " Cursor re-positioning of d-special in case 'startofline' is 0.
        if &startofline ==# 0 && l:need_repos && l:result_dict["visualmode"] ==# "V"
            if has("patch-8.2.5034") || has("nvim")
                call cursor(0, virtcol2col(0, line("."), l:orig_curpos[4]))
            else
                execute "normal! " . l:orig_curpos[4] . "|"
                call cursor(0, col("."))
            endif
        endif

        " Special treatment to |c| which needs to drop the user in insert mode.
        if a:operator ==# "c" && a:repeat == 0
            if l:result_dict["cursor"][2] >= col("$")
                if exists("$JIEBA_TEST_CASE")
                    normal! A
                else
                    startinsert!
                endif
            else
                if exists("$JIEBA_TEST_CASE")
                    normal! i
                else
                    startinsert
                endif
            endif
        endif
    endif
endfunction

function! s:JiebaNmap_ky(ky, count)
    call JiebaNmap(a:ky, a:count, "")
endfunction

" Keep in one line to help debug. There is no readability anyway even if this
" is properly wrapped to 80 width. Same below.
for ky in s:motions
    execute 'nnoremap <expr> <silent> <Plug>(Jieba_' . ky . ') ":<C-u>call <SID>JiebaNmap_ky(' . "'" . ky . "', v:count1" . ')<CR>"'
endfor

function! s:JiebaXmap_ky(ky, count)
    call JiebaXmap(a:ky, a:count, "")
endfunction

for ky in s:motions + s:objects
    execute 'xnoremap <expr> <silent> <Plug>(Jieba_' . ky . ') ":<C-u>call <SID>JiebaXmap_ky(' . "'" . ky . "', v:count1" . ')<CR>"'
endfor

function! s:JiebaOmap_internal_ky(ky, count, operator, register)
    silent! call repeat#setreg("\<Plug>(Jieba_internal_o_" . a:ky . ")", a:register)
    call JiebaOmap(a:ky, 1, a:count, a:operator, a:register, "")
    silent! call repeat#set("\<Plug>(Jieba_internal_o_" . a:ky . ")", a:count)
endfunction

function! s:JiebaOmap_ky(ky, count, operator, register)
    silent! call repeat#setreg("\<Plug>(Jieba_internal_o_" . a:ky . ")", a:register)
    call JiebaOmap(a:ky, 0, a:count, a:operator, a:register, "")
    silent! call repeat#set("\<Plug>(Jieba_internal_o_" . a:ky . ")", a:count)
endfunction

for ky in s:motions + s:objects
    execute "nnoremap <expr> <silent> <Plug>(Jieba_internal_o_" . ky . ") " . '":<C-u>call <SID>JiebaOmap_internal_ky(' . "'" . ky . "'" . ', " . v:count1 . ", ' . "'" . '" . v:operator . "' . "'" . ', ' . "'" . '" . v:register . "' . "'" . ')<CR>"'
    execute "onoremap <expr> <silent> <Plug>(Jieba_" . ky . ") " . '"<Esc>:<C-u>call <SID>JiebaOmap_ky(' . "'" . ky . "'" . ', " . v:count1 . ", ' . "'" . '" . v:operator . "' . "'" . ', ' . "'" . '" . v:register . "' . "'" . ')<CR>"'
endfor

let s:modes = ["n", "x", "o"]
if g:jieba_vim_keymap
    for ky in s:motions
        for md in s:modes
            execute md . "map " . ky . " <Plug>(Jieba_" . ky . ")"
        endfor
    endfor
    for ky in s:objects
        for md in s:modes
            if md !=# "n"
                execute md . "map " . ky . " <Plug>(Jieba_" . ky . ")"
            endif
        endfor
    endfor
endif

function s:UpdateIsk()
    if has("nvim")
        lua jieba_vim:update_isk(vim.o.iskeyword)
    else
        py3 jieba_vim.navigation.update_isk(vim.eval('&iskeyword'))
    endif
endfunction

augroup jieba_vim_update_isk
    autocmd!
    autocmd OptionSet iskeyword call s:UpdateIsk()
augroup END


" Reference: https://github.com/junegunn/fzf/blob/master/plugin/fzf.vim
function! jieba_vim#install()
    if s:is_win && !has("win32unix")
        let l:script = s:base_dir . "/build.ps1"
        let l:script = "powershell -ExecutionPolicy Bypass -file " . shellescape(l:script)
    else
        let l:script = s:base_dir . "/build.sh"
    endif
    if has("nvim")
        let $JIEBA_VIM_INSTALL_NVIM = "1"
    endif
    call system(l:script)
    if v:shell_error
        throw "Failed to run build script: " . l:script
    endif
    call s:CheckCdylib()
    let s:loaded_jieba_vim_word_motion = 0
    call s:InitWordMotion()
endfunction
