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


" Reference: https://github.com/junegunn/fzf/blob/master/plugin/fzf.vim
let s:is_win = has("win32") || has("win64")
if s:is_win && &shellslash
    set noshellslash
    let s:base_dir = expand("<sfile>:h:h:h")
    set shellslash
else
    let s:base_dir = expand("<sfile>:h:h:h")
endif

if s:is_win && !has("win32unix") && has("nvim")
    let s:cdylib_suffix = ".dll"
elseif s:is_win && !has("win32unix")
    let s:cdylib_suffix = ".pyd"
else
    let s:cdylib_suffix = ".so"
endif

function! jieba_vim#loader#check_cdylib() abort
    let g:jieba_vim_loaded_cdylib = get(g:, "jieba_vim_loaded_cdylib", 0)
    if g:jieba_vim_loaded_cdylib
        return
    endif
    if has("nvim")
        if filereadable(s:base_dir . "/lua/jieba_vim/jieba_vim_rs" . s:cdylib_suffix)
            lua jieba_vim = require("jieba_vim")
            let g:jieba_vim_loaded_cdylib = 1
        else
            let g:jieba_vim_loaded_cdylib = 0
        endif
    else
        if filereadable(s:base_dir . "/pythonx/jieba_vim/jieba_vim_rs" . s:cdylib_suffix)
            py3 import jieba_vim.navigation
            let g:jieba_vim_loaded_cdylib = 1
        else
            let g:jieba_vim_loaded_cdylib = 0
        endif
    endif
endfunction

function! jieba_vim#loader#init_word_motion() abort
    let g:jieba_vim_loaded_word_motion = get(g:, "jieba_vim_loaded_word_motion", 0)
    if g:jieba_vim_loaded_word_motion
        return
    endif
    if !g:jieba_vim_loaded_cdylib
        return
    endif
    let l:args = [g:jieba_vim_user_dict, &iskeyword, str2nr(g:jieba_vim_lazy)]
    if has("nvim")
        let l:init_word_motion_err = luaeval("jieba_vim:init_word_motion(unpack(_A))", l:args)
        if l:init_word_motion_err !=# ""
            echoerr l:init_word_motion_err
            return
        endif
    else
        let l:init_word_motion_err = py3eval(
            \ "jieba_vim.navigation.init_word_motion(*vim.eval('l:args'))")
        if l:init_word_motion_err !=# "" && l:init_word_motion_err !=# v:none
            echoerr l:init_word_motion_err
            return
        endif
    endif
    let g:jieba_vim_loaded_word_motion = 1
endfunction

" Reference: https://github.com/junegunn/fzf/blob/master/plugin/fzf.vim
function! jieba_vim#loader#install()
    if s:is_win && !has("win32unix")
        let l:script = s:base_dir . "/build.ps1"
        let l:script = "powershell -ExecutionPolicy Bypass -file " . shellescape(l:script)
    else
        let l:script = s:base_dir . "/build.sh"
    endif
    if has("nvim")
        let $JIEBA_VIM_INSTALL_NVIM = "1"
    endif
    let g:jieba_vim_build_error = system(l:script)
    if v:shell_error
        throw "jieba_vim#install: build script " . l:script
            \ . " returns " . v:shell_error
            \ . " (see g:jieba_vim_build_error)"
    else
        unlet g:jieba_vim_build_error
    endif
    unlet! g:jieba_vim_loaded_cdylib
    unlet! g:jieba_vim_loaded_word_motion
    call jieba_vim#loader#check_cdylib()
    call jieba_vim#loader#init_word_motion()
endfunction

function! jieba_vim#loader#ensure_loaded() abort
    call jieba_vim#loader#check_cdylib()
    call jieba_vim#loader#init_word_motion()
    if !g:jieba_vim_loaded_cdylib 
        throw "cdylib unloaded; run jieba_vim#install() first"
    endif
    if !g:jieba_vim_loaded_word_motion
        throw "word_motion uninitialized; check jieba_vim config"
    endif
endfunction
