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
" TODO
let g:jieba_vim_preview_limits = get(g:, "jieba_vim_preview_limits", 0)

if !has("nvim") && !has('python3')
    echoerr "python3 is required by jieba.vim"
    finish
endif


""
" 取消按词跳转位置预览
command! JiebaPreviewCancel call jieba_vim#mapping#preview_cancel()

let s:motions = ["w", "W", "e", "E", "b", "B", "ge", "gE"]
let s:objects = ["iw", "iW", "aw", "aW"]


for ky in s:motions
    execute 'nnoremap <silent> <Plug>(Jieba_preview_' . ky . ') :<C-u>call jieba_vim#mapping#preview("' . ky . '")<CR>'
endfor
nnoremap <silent> <Plug>(Jieba_preview_cancel) :<C-u>call jieba_vim#mapping#preview_cancel()<CR>


for ky in s:motions
    execute 'nnoremap <expr> <silent> <Plug>(Jieba_' . ky . ') jieba_vim#mapping#nmap_expr("' . ky . '", "")'
endfor
nnoremap <expr> <silent> <Plug>(Jieba_C_Left) jieba_vim#mapping#nmap_expr("\<C-Left>", "")
nnoremap <expr> <silent> <Plug>(Jieba_S_Left) jieba_vim#mapping#nmap_expr("\<S-Left>", "")
nnoremap <expr> <silent> <Plug>(Jieba_C_Right) jieba_vim#mapping#nmap_expr("\<C-Right>", "")
nnoremap <expr> <silent> <Plug>(Jieba_S_Right) jieba_vim#mapping#nmap_expr("\<S-Right>", "")

for ky in s:motions + s:objects
    execute 'xnoremap <expr> <silent> <Plug>(Jieba_' . ky . ') jieba_vim#mapping#xmap_expr("' . ky . '", "")'
endfor
xnoremap <expr> <silent> <Plug>(Jieba_C_Left) jieba_vim#mapping#xmap_expr("\<C-Left>", "")
xnoremap <expr> <silent> <Plug>(Jieba_S_Left) jieba_vim#mapping#xmap_expr("\<S-Left>", "")
xnoremap <expr> <silent> <Plug>(Jieba_C_Right) jieba_vim#mapping#xmap_expr("\<C-Right>", "")
xnoremap <expr> <silent> <Plug>(Jieba_S_Right) jieba_vim#mapping#xmap_expr("\<S-Right>", "")


for ky in s:motions + s:objects
    execute 'onoremap <expr> <silent> <Plug>(Jieba_internal_o_' . ky . ') jieba_vim#mapping#omap_playback_expr("' . ky . '", 1, "")'
    execute 'onoremap <expr> <silent> <Plug>(Jieba_' . ky . ') jieba_vim#mapping#omap_expr("' . ky . '", "")'
endfor
onoremap <expr> <silent> <Plug>(Jieba_internal_o_C_Left) jieba_vim#mapping#omap_playback_expr("\<C-Left>", 1, "")
onoremap <expr> <silent> <Plug>(Jieba_internal_o_S_Left) jieba_vim#mapping#omap_playback_expr("\<S-Left>", 1, "")
onoremap <expr> <silent> <Plug>(Jieba_internal_o_C_Right) jieba_vim#mapping#omap_playback_expr("\<C-Right>", 1, "")
onoremap <expr> <silent> <Plug>(Jieba_internal_o_S_Right) jieba_vim#mapping#omap_playback_expr("\<S-Right>", 1, "")
onoremap <expr> <silent> <Plug>(Jieba_C_Left) jieba_vim#mapping#omap_expr("\<C-Left>", "")
onoremap <expr> <silent> <Plug>(Jieba_S_Left) jieba_vim#mapping#omap_expr("\<S-Left>", "")
onoremap <expr> <silent> <Plug>(Jieba_C_Right) jieba_vim#mapping#omap_expr("\<C-Right>", "")
onoremap <expr> <silent> <Plug>(Jieba_S_Right) jieba_vim#mapping#omap_expr("\<S-Right>", "")

inoremap <expr> <silent> <Plug>(Jieba_C_w) jieba_vim#mapping#imap_expr("\<C-w>", "")
inoremap <expr> <silent> <Plug>(Jieba_C_Left) jieba_vim#mapping#imap_expr("\<C-Left>", "")
inoremap <expr> <silent> <Plug>(Jieba_S_Left) jieba_vim#mapping#imap_expr("\<S-Left>", "")
inoremap <expr> <silent> <Plug>(Jieba_C_Right) jieba_vim#mapping#imap_expr("\<C-Right>", "")
inoremap <expr> <silent> <Plug>(Jieba_S_Right) jieba_vim#mapping#imap_expr("\<S-Right>", "")

let s:modes = ["n", "x", "o"]
if g:jieba_vim_keymap
    for ky in s:motions
        for md in s:modes
            execute md . "map " . ky . " <Plug>(Jieba_" . ky . ")"
        endfor
    endfor
    for md in s:modes
        execute md . "map <C-Left> <Plug>(Jieba_C_Left)"
        execute md . "map <S-Left> <Plug>(Jieba_S_Left)"
        execute md . "map <C-Right> <Plug>(Jieba_C_Right)"
        execute md . "map <S-Right> <Plug>(Jieba_S_Right)"
    endfor
    for ky in s:objects
        for md in s:modes
            if md !=# "n"
                execute md . "map " . ky . " <Plug>(Jieba_" . ky . ")"
            endif
        endfor
    endfor
    imap <C-w> <Plug>(Jieba_C_w)
endif


augroup jieba_vim_update_isk
    autocmd!
    autocmd OptionSet iskeyword call jieba_vim#utils#update_isk()
augroup END


function! jieba_vim#install()
    call jieba_vim#loader#install()
endfunction
