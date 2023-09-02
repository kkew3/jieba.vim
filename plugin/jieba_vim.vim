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

nnoremap <silent> <Plug>(Jieba_preview_b) :<C-u>py3 jieba_vim.preview(jieba_vim.pysrc.navigation.backward_word_start)<CR>
nnoremap <silent> <Plug>(Jieba_preview_B) :<C-u>py3 jieba_vim.preview(jieba_vim.pysrc.navigation.backward_WORD_start)<CR>
nnoremap <silent> <Plug>(Jieba_preview_ge) :<C-u>py3 jieba_vim.preview(jieba_vim.pysrc.navigation.backward_word_end)<CR>
nnoremap <silent> <Plug>(Jieba_preview_gE) :<C-u>py3 jieba_vim.preview(jieba_vim.pysrc.navigation.backward_WORD_end)<CR>
nnoremap <silent> <Plug>(Jieba_preview_w) :<C-u>py3 jieba_vim.preview(jieba_vim.pysrc.navigation.forward_word_start)<CR>
nnoremap <silent> <Plug>(Jieba_preview_W) :<C-u>py3 jieba_vim.preview(jieba_vim.pysrc.navigation.forward_WORD_start)<CR>
nnoremap <silent> <Plug>(Jieba_preview_e) :<C-u>py3 jieba_vim.preview(jieba_vim.pysrc.navigation.forward_word_end)<CR>
nnoremap <silent> <Plug>(Jieba_preview_E) :<C-u>py3 jieba_vim.preview(jieba_vim.pysrc.navigation.forward_WORD_end)<CR>
nnoremap <silent> <Plug>(Jieba_preview_cancel) :<C-u>py3 jieba_vim.preview_cancel()<CR>

nnoremap <silent> <Plug>(Jieba_b) :<C-u>py3 jieba_vim.navigation.backward_word_start()<CR>
onoremap <silent> <plug>(Jieba_b) :<C-u>py3 jieba_vim.navigation.backward_word_start()<CR>
vnoremap <silent> <plug>(Jieba_b) :<C-u>py3 jieba_vim.navigation.backward_word_start_v()<CR>

nnoremap <silent> <Plug>(Jieba_B) :<C-u>py3 jieba_vim.navigation.backward_WORD_start()<CR>
onoremap <silent> <Plug>(Jieba_B) :<C-u>py3 jieba_vim.navigation.backward_WORD_start()<CR>
vnoremap <silent> <Plug>(Jieba_B) :<C-u>py3 jieba_vim.navigation.backward_WORD_start_v()<CR>

nnoremap <silent> <Plug>(Jieba_ge) :<C-u>py3 jieba_vim.navigation.backward_word_end()<CR>
onoremap <silent> <Plug>(Jieba_ge) :<C-u>py3 jieba_vim.navigation.backward_word_end()<CR>
vnoremap <silent> <Plug>(Jieba_ge) :<C-u>py3 jieba_vim.navigation.backward_word_end_v()<CR>

nnoremap <silent> <Plug>(Jieba_gE) :<C-u>py3 jieba_vim.navigation.backward_WORD_end()<CR>
onoremap <silent> <Plug>(Jieba_gE) :<C-u>py3 jieba_vim.navigation.backward_WORD_end()<CR>
vnoremap <silent> <Plug>(Jieba_gE) :<C-u>py3 jieba_vim.navigation.backward_WORD_end_v()<CR>

nnoremap <silent> <Plug>(Jieba_w) :<C-u>py3 jieba_vim.navigation.forward_word_start()<CR>
onoremap <silent> <Plug>(Jieba_w) :<C-u>py3 jieba_vim.navigation.forward_word_start()<CR>
vnoremap <silent> <Plug>(Jieba_w) :<C-u>py3 jieba_vim.navigation.forward_word_start_v()<CR>

nnoremap <silent> <Plug>(Jieba_W) :<C-u>py3 jieba_vim.navigation.forward_WORD_start()<CR>
onoremap <silent> <Plug>(Jieba_W) :<C-u>py3 jieba_vim.navigation.forward_WORD_start()<CR>
vnoremap <silent> <Plug>(Jieba_W) :<C-u>py3 jieba_vim.navigation.forward_WORD_start_v()<CR>

nnoremap <silent> <Plug>(Jieba_e) :<C-u>py3 jieba_vim.navigation.forward_word_end()<CR>
onoremap <silent> <Plug>(Jieba_e) :<C-u>py3 jieba_vim.navigation.forward_word_end()<CR>
vnoremap <silent> <Plug>(Jieba_e) :<C-u>py3 jieba_vim.navigation.forward_word_end_v()<CR>

nnoremap <silent> <Plug>(Jieba_E) :<C-u>py3 jieba_vim.navigation.forward_WORD_end()<CR>
onoremap <silent> <Plug>(Jieba_E) :<C-u>py3 jieba_vim.navigation.forward_WORD_end()<CR>
vnoremap <silent> <Plug>(Jieba_E) :<C-u>py3 jieba_vim.navigation.forward_WORD_end_v()<CR>
