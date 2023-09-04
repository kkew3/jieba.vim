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
py3 import jieba_vim.pysrc.navigation


command! JiebaInit py3 jieba_vim.jieba_initialize()
command! JiebaPreviewCancel py3 jieba_vim.preview_cancel()

let s:motions = ["b", "B", "ge", "gE", "w", "W", "e", "E",]

for ky in s:motions
    execute 'nnoremap <silent> <Plug>(Jieba_preview_' . ky . ') :<C-u>py3 jieba_vim.preview(jieba_vim.pysrc.navigation.wordmotion_' . ky . ')<CR>'
endfor
nnoremap <silent> <Plug>(Jieba_preview_cancel) :<C-u>py3 jieba_vim.preview_cancel()<CR>

let s:modes = ["n", "o", "v",]
for ky in s:motions
    for md in s:modes
        if md ==# "v"
            execute 'vnoremap <silent> <Plug>(Jieba_' . ky . ') :<C-u>py3 jieba_vim.navigation.wordmotion_' . ky . '_v()<CR>'
        else
            execute md . 'noremap <silent> <Plug>(Jieba_' . ky . ') :<C-u>py3 jieba_vim.navigation.wordmotion_' . ky . '()<CR>'
        endif
    endfor
endfor
