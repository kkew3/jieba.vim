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


function jieba_vim#mapping#preview_cancel()
    execute "hi clear JiebaPreview"
endfunction

function! jieba_vim#mapping#preview(motion)
    let l:limit = g:jieba_vim_preview_limits
    if l:limit < 0
        let l:limit = 99999
    endif
    let l:cursor_positions = jieba_vim#model#preview(a:motion, getcurpos(), l:limit)
    if empty(l:cursor_positions)
        call jieba_vim#mapping#preview_cancel()
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

function! jieba_vim#mapping#nmap(motion, count, model_funcname)
    if a:model_funcname !=# ""
        let l:result_dict = function(a:model_funcname)(a:motion, getcurpos(), a:count)
    else
        let l:result_dict = jieba_vim#model#nmap(a:motion, getcurpos(), a:count)
    endif
    call cursor(l:result_dict["cursor"][1:2])
    if l:result_dict["prevent_change"]
        call jieba_vim#utils#consume_chars()
    endif
endfunction

function! jieba_vim#mapping#nmap_expr(motion, model_funcname)
    if a:motion ==# "\<C-Left>"
        let l:equiv_motion = "B"
    elseif a:motion ==# "\<S-Left>"
        let l:equiv_motion = "b"
    elseif a:motion ==# "\<C-Right>"
        let l:equiv_motion = "W"
    elseif a:motion ==# "\<S-Right>"
        let l:equiv_motion = "w"
    else
        let l:equiv_motion = a:motion
    endif
    return "\<Cmd>call jieba_vim#mapping#nmap('" . l:equiv_motion . "', v:count1, '" . a:model_funcname . "')\<CR>"
endfunction


function! jieba_vim#mapping#xmap(motion, count, model_funcname)
    noautocmd execute "normal! \<Esc>"
    let l:orig_mark_a = getpos("'a")
    let l:orig_mark_b = getpos("'b")
    noautocmd silent execute "normal! gvomaomb\<Esc>"
    let l:visual_begin = getpos("'a")
    let l:visial_end = getpos("'b")
    call setpos("'a", l:orig_mark_a)
    call setpos("'b", l:orig_mark_b)
    let l:vmode = visualmode()
    if a:model_funcname !=# ""
        let l:result_dict = function(a:model_funcname)(l:vmode, a:motion, l:visual_begin, l:visial_end, a:count)
    else
        let l:result_dict = jieba_vim#model#xmap(l:vmode, a:motion, l:visual_begin, l:visial_end, a:count)
    endif
    noautocmd execute "normal! " . l:result_dict["visualmode"] . "\<Esc>"
    call setpos("'<", l:result_dict["langle"])
    call setpos("'>", l:result_dict["rangle"])
    if l:result_dict["visualmode"] ==# "v" && l:vmode !=# "v"
        " Release a ModeChanged event when the visualmode did change.
        normal! gv
    else
        noautocmd normal! gv
    endif
    if l:result_dict["prevent_change"]
        call jieba_vim#utils#consume_chars()
    endif
endfunction

function! jieba_vim#mapping#xmap_expr(motion, model_funcname)
    if a:motion ==# "\<C-Left>"
        let l:equiv_motion = "B"
    elseif a:motion ==# "\<S-Left>"
        let l:equiv_motion = "b"
    elseif a:motion ==# "\<C-Right>"
        let l:equiv_motion = "W"
    elseif a:motion ==# "\<S-Right>"
        let l:equiv_motion = "w"
    else
        let l:equiv_motion = a:motion
    endif
    return "\<Cmd>call jieba_vim#mapping#xmap('" . l:equiv_motion . "', v:count1, '" . a:model_funcname . "')\<CR>"
endfunction

function! s:omap_playback_core(motion, repeat, count, operator, register, model_funcname)
    let l:orig_curpos = getcurpos()
    if type(a:model_funcname) == v:t_string
        if a:model_funcname !=# ""
            let l:result_dict = function(a:model_funcname)(a:motion, l:orig_curpos, a:count, a:operator)
        else
            let l:result_dict = jieba_vim#model#omap(a:motion, l:orig_curpos, a:count, a:operator)
        endif
    else
        let l:result_dict = a:model_funcname
    endif
    call cursor(l:result_dict["langle"][1:2])

    if l:result_dict["prevent_change"]
        " Land the cursor to potentially a new position.
        call cursor(l:result_dict["cursor"][1:2])
        call jieba_vim#utils#consume_chars()
    else
        if a:operator !=# "y"
            " This no-op line effectively sets an undoable checkpoint such that
            " |u| undos all operations up to this line.
            call setline(".", getline("."))
        endif

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
        if jieba_vim#utils#is_forward_motion(a:motion)
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

function! jieba_vim#mapping#omap_playback(motion, repeat, count, operator, register, model_funcname)
    let l:orig_curpos = getcurpos()
    if a:model_funcname !=# ""
        let l:result_dict = function(a:model_funcname)(a:motion, l:orig_curpos, a:count, a:operator)
    else
        let l:result_dict = jieba_vim#model#omap(a:motion, l:orig_curpos, a:count, a:operator)
    endif
    if !l:result_dict["prevent_change"] && a:operator !=# "y"
        silent! call repeat#setreg(a:operator . "\<Plug>(Jieba_internal_o_" . a:motion . ")", a:register)
    endif
    call s:omap_playback_core(a:motion, a:repeat, a:count, a:operator, a:register, l:result_dict)
    if !l:result_dict["prevent_change"] && a:operator !=# "y"
        silent! call repeat#set(a:operator . "\<Plug>(Jieba_internal_o_" . a:motion . ")", a:count)
    endif
endfunction

function! jieba_vim#mapping#omap_playback_expr(motion, repeat, model_funcname)
    if a:motion ==# "\<C-Left>"
        let l:equiv_motion = "B"
    elseif a:motion ==# "\<S-Left>"
        let l:equiv_motion = "b"
    elseif a:motion ==# "\<C-Right>"
        let l:equiv_motion = "W"
    elseif a:motion ==# "\<S-Right>"
        let l:equiv_motion = "w"
    else
        let l:equiv_motion = a:motion
    endif
    return "\<Esc>\<Cmd>call jieba_vim#mapping#omap_playback('" . l:equiv_motion . "', " . a:repeat . ", " . v:count1 . ", '" . v:operator . "', '" . v:register . "', '" . a:model_funcname . "')\<CR>"
endfunction

function! jieba_vim#mapping#omap_expr(motion, model_funcname)
    return jieba_vim#mapping#omap_playback_expr(a:motion, 0, a:model_funcname)
endfunction

function! s:imap_ctrlw_expr(model_funcname)
    " l:curpos: [_, lnum, col, off, _]
    let l:curpos = getcurpos()
    if exists("$JIEBA_TEST_CASE")
        if a:model_funcname !=# ""
            let l:result_dict = function(a:model_funcname)("\<C-w>", l:curpos)
        else
            let l:result_dict = jieba_vim#model#imap("\<C-w>", l:curpos)
        endif
    endif
    if l:curpos[3] > 0
        return "\<Cmd>call cursor(0," . l:curpos[2] . ",0)\<CR>"
    elseif l:curpos[2] ==# 1
        return "\<BS>"
    else
        if !exists("$JIEBA_TEST_CASE")
            if a:model_funcname !=# ""
                let l:result_dict = function(a:model_funcname)("\<C-w>", l:curpos)
            else
                let l:result_dict = jieba_vim#model#imap("\<C-w>", l:curpos)
            endif
        endif
        return "\<Cmd>call jieba_vim#utils#del_to_cursor("
            \ . l:result_dict["cursor"][2] . ","
            \ . l:curpos[2] . ")\<CR>"
    endif
endfunction

function! s:imap_arrow_expr(motion, model_funcname)
    let l:curpos = getcurpos()
    if a:model_funcname !=# ""
        let l:result_dict = function(a:model_funcname)(a:motion, l:curpos)
    else
        let l:result_dict = jieba_vim#model#imap(a:motion, l:curpos)
    endif
    return "\<Cmd>call cursor("
        \ . l:result_dict["cursor"][1] . ","
        \ . l:result_dict["cursor"][2] . ")\<CR>"
endfunction

function! jieba_vim#mapping#imap_expr(motion, model_funcname)
    if a:motion ==# "\<C-w>"
        return s:imap_ctrlw_expr(a:model_funcname)
    else
        return s:imap_arrow_expr(a:motion, a:model_funcname)
    endif
endfunction
