command! Space normal! a·
command! Return normal! a␊
command! Empty normal! a␀

nnoremap g. :Space<CR>
nnoremap g; :Return<CR>
nnoremap g0 :Empty<CR>

setlocal commentstring=//\ %s
setlocal tabstop=2
setlocal softtabstop=2
setlocal shiftwidth=2
setlocal expandtab

if executable("jieba_vim_rs_metatest")
    command! Fix write | call system('jieba_vim_rs_metatest check --fix ' . expand("%:p:S")) | checktime
    set makeprg=jieba_vim_rs_metatest\ check\ %
    let &errorformat = 'Error:\\ parsing\\ error:\\ %f:%l:%c:\\ %m,'
            \ . 'Error:\\ parsing\\ error:\\ %f:%l-%e:\\ %m,'
            \ . 'Error:\\ parsing\\ error:\\ %f:%l:\\ %m,'
            \ . 'Error:\\ parsing\\ error:\\ %f:\\ %m,'
            \ . '%f:%l:%c:\\ %m,'
            \ . '%f:%l-%e:\\ %m,'
            \ . '%f:%l:\\ %m,'
            \ . '%f:\\ %m,'
            \ . '%-G%.%#'
endif
