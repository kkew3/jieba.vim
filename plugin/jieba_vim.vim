if exists("g:loaded_jieba_vim")
    finish
endif
let g:loaded_jieba_vim = 1

if !has('python3')
    echoerr "python3 is required by jieba.vim"
    finish
endif

py3 import jieba_vim
py3 import jieba_vim.navigation
py3 import jieba_vim.jieba_navi_rs


command! JiebaPreviewCancel py3 jieba_vim.preview_cancel()

let s:motions = ["b", "B", "ge", "gE", "w", "W", "e", "E",]

for ky in s:motions
    execute 'nnoremap <silent> <Plug>(Jieba_preview_' . ky . ') :<C-u>py3 jieba_vim.preview(jieba_vim.jieba_navi_rs.wordmotion_' . ky . ')<CR>'
endfor
nnoremap <silent> <Plug>(Jieba_preview_cancel) :<C-u>py3 jieba_vim.preview_cancel()<CR>

let s:modes = ["n", "o", "x",]
for ky in s:motions
    for md in s:modes
        if md ==# "x"
            " Reference: https://github.com/svermeulen/vim-NotableFt/blob/master/plugin/NotableFt.vim
            execute 'xnoremap <expr> <silent> <Plug>(Jieba_' . ky . ') "<Esc>:<C-u>py3 jieba_vim.navigation.wordmotion_' . ky . '(" . v:count1 . ")<CR>m>gv"'
        else
            execute md . 'noremap <expr> <silent> <Plug>(Jieba_' . ky . ') ":<C-u>py3 jieba_vim.navigation.wordmotion_' . ky . '(" . v:count1 . ")<CR>"'
        endif
    endfor
endfor
