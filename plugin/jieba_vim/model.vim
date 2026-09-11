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


function jieba_vim#model#preview(...)
    call jieba_vim#loader#ensure_loaded()

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

function! jieba_vim#model#nmap(...)
    call jieba_vim#loader#ensure_loaded()

    if has("nvim")
        return luaeval("jieba_vim:nmap(jieba_vim.buffer, unpack(_A))", a:000)
    else
        return py3eval(
            \ "jieba_vim.navigation.nmap(vim.current.buffer, *vim.eval('a:000'))")
    endif
endfunction

function! jieba_vim#model#xmap(...)
    call jieba_vim#loader#ensure_loaded()

    if has("nvim")
        return luaeval("jieba_vim:xmap(jieba_vim.buffer, unpack(_A))", a:000)
    else
        return py3eval(
            \ "jieba_vim.navigation.xmap(vim.current.buffer, *vim.eval('a:000'))")
    endif
endfunction

function! s:raw_model_omap(...)
    call jieba_vim#loader#ensure_loaded()

    if has("nvim")
        return luaeval("jieba_vim:omap(jieba_vim.buffer, unpack(_A))", a:000)
    else
        return py3eval(
            \ "jieba_vim.navigation.omap(vim.current.buffer, *vim.eval('a:000'))")
    endif
endfunction

function jieba_vim#model#omap(...)
    let l:result_dict = call("<SID>raw_model_omap", a:000)
    " Check if we are selecting an empty region.
    if l:result_dict["langle"] ==# l:result_dict["rangle"]
        \ && l:result_dict["selection"] ==# "exclusive"
        \ && l:result_dict["visualmode"] !=# "V"
        \ && !l:result_dict["prevent_change"]
        \ && stridx(&cpoptions, "E") >= 0
        let l:result_dict["prevent_change"] = 1
    endif
    return l:result_dict
endfunction

function! jieba_vim#model#imap(...)
    call jieba_vim#loader#ensure_loaded()

    if has("nvim")
        return luaeval("jieba_vim:imap(jieba_vim.buffer, unpack(_A))", a:000)
    else
        return py3eval(
            \ "jieba_vim.navigation.imap(vim.current.buffer, *vim.eval('a:000'))")
    endif
endfunction
