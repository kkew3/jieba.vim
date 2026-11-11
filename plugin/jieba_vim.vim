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

""
" (默认 0)：motion 预览上限。若为正数，预览该次数内 motion 后的光标位置；
" 若为 0，预览当前行；否则预览 99999 次内 motion 后的光标位置。
let g:jieba_vim_preview_limits = get(g:, "jieba_vim_preview_limits", 0)

""
" (默认 0)：是/否 (1/0) 启用对 |g@| 的实验性支持。
let g:jieba_vim_experimental_opfunc = get(g:, "jieba_vim_experimental_opfunc", 0)

""
" (默认 0)：是/否 (1/0) 启用高亮光标下 token。
let g:jieba_vim_highlight_current_token = get(g:, "jieba_vim_highlight_current_token", 0)

if !has("nvim") && !has('python3')
    echoerr "python3 is required by jieba.vim"
    finish
endif


""
" 取消按词跳转位置预览
command! JiebaPreviewCancel call jieba_vim#mapping#preview_cancel()


function! s:escape_key(key, ...)
    if stridx(a:key, "_") < 0
        return a:key
    endif
    if a:0 && a:1
        return eval('"\<' . substitute(a:key, "_", "-", "") . '>"')
    endif
    return "<" . substitute(a:key, "_", "-", "") . ">"
endfunction

let s:motions = ["w", "W", "e", "E", "b", "B", "ge", "gE"]
let s:objects = ["iw", "iW", "aw", "aW"]
let s:arrows = ["C_Left", "S_Left", "C_Right", "S_Right"]
let s:ispecial = ["C_w"]


for ky in s:motions
    execute 'nnoremap <silent> <Plug>(Jieba_preview_' . ky . ') '
        \ . ':<C-u>call jieba_vim#mapping#preview(' . string(ky) . ')<CR>'
endfor
nnoremap <silent> <Plug>(Jieba_preview_cancel) :<C-u>call jieba_vim#mapping#preview_cancel()<CR>

for ky in s:motions + s:arrows
    execute 'nnoremap <expr> <silent> <Plug>(Jieba_' . ky . ') '
        \ . 'jieba_vim#mapping#nmap_expr(' . string(s:escape_key(ky, 1)) . ', "")'
endfor

for ky in s:motions + s:objects + s:arrows
    execute 'xnoremap <expr> <silent> <Plug>(Jieba_' .ky . ') '
        \ . 'jieba_vim#mapping#xmap_expr(' . string(s:escape_key(ky, 1)) . ', "")'
endfor

for ky in s:motions + s:objects + s:arrows
    execute 'onoremap <expr> <silent> <Plug>(Jieba_internal_o_' . ky . ') '
        \ . 'jieba_vim#mapping#omap_playback_expr(' . string(s:escape_key(ky, 1)) . ', 1, "")'
    execute 'onoremap <expr> <silent> <Plug>(Jieba_' . ky . ') '
        \ . 'jieba_vim#mapping#omap_expr(' . string(s:escape_key(ky, 1)) . ', "")'
endfor

for ky in s:arrows + s:ispecial
    execute 'inoremap <expr> <silent> <Plug>(Jieba_' . ky . ') '
        \ . 'jieba_vim#mapping#imap_expr(' . string(s:escape_key(ky, 1)) . ', "")'
endfor

cnoremap <expr> <Plug>(Jieba_C_r_C_w) jieba_vim#mapping#cmap_ctrlr_expr(1, "")
cnoremap <expr> <Plug>(Jieba_C_r_C_a) jieba_vim#mapping#cmap_ctrlr_expr(0, "")

function! jieba_vim#default_keymap()
    for ky in s:motions + s:arrows
        execute "nmap " . s:escape_key(ky) . " <Plug>(Jieba_" . ky . ")"
        execute "xmap " . s:escape_key(ky) . " <Plug>(Jieba_" . ky . ")"
        execute "omap " . s:escape_key(ky) . " <Plug>(Jieba_" . ky . ")"
    endfor
    for ky in s:objects
        execute "xmap " . s:escape_key(ky) . " <Plug>(Jieba_" . ky . ")"
        execute "omap " . s:escape_key(ky) . " <Plug>(Jieba_" . ky . ")"
    endfor
    for ky in s:ispecial
        execute "imap " . s:escape_key(ky) . " <Plug>(Jieba_" . ky . ")"
    endfor
endfunction

if g:jieba_vim_keymap
    call jieba_vim#default_keymap()
endif


augroup jieba_vim_update_isk
    autocmd!
    autocmd OptionSet iskeyword call jieba_vim#utils#update_isk()
augroup END


hi default link JiebaHighlightCurrentToken ColorColumn
hi default link JiebaHighlightCurrentBigToken CursorColumn

augroup jieba_vim_highlight_current_token
    autocmd!
    autocmd BufLeave,WinLeave * call jieba_vim#utils#clear_current_token_highlighting()
    autocmd CursorMoved,InsertLeave * call jieba_vim#utils#update_current_token_highlighting()
    autocmd ModeChanged n:* call jieba_vim#utils#clear_current_token_highlighting()
augroup END


function! jieba_vim#install()
    call jieba_vim#loader#install()
endfunction


function! jieba_vim#cword(...)
    let l:word = a:0 ? str2nr(a:1) : 1
    let l:result_dict = jieba_vim#model#cword(getcurpos(), l:word)
    if l:result_dict["prevent_change"]
        return ""
    endif
    let l:line = getline(".")
    " -1: first_char indexed from 1 whereas string indexed from 0;
    " -2: same as above; furthermore, Vim string slice is doubly inclusive.
    return l:line[l:result_dict["first_char"] - 1 : l:result_dict["last_char1"] - 2]
endfunction

function! jieba_vim#ctoken(...)
    let l:word = a:0 ? str2nr(a:1) : 1
    let l:result_dict = jieba_vim#model#ctoken(getcurpos(), l:word)
    if l:result_dict["prevent_change"]
        return ""
    endif
    let l:line = getline(".")
    return l:line[l:result_dict["first_char"] - 1 : l:result_dict["last_char1"] - 2]
endfunction
