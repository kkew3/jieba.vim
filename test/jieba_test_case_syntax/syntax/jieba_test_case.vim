if exists("b:current_syntax")
    finish
endif

runtime! syntax/jinja.vim
if exists("b:current_syntax")
    unlet b:current_syntax
endif

" Comments
syntax match jiebaTestCaseComment /^\/\/.*/

" Head conditionals
syntax match jiebaTestCaseHeadConditional /^\%(?\s!\?has:\|?\sversion:\)/

syntax match jiebaTestCaseResetDefaults /^##.*$/

syntax match jiebaTestCaseGlobalDirective /^#\(V\|M\|X\|O\|K\|C\|R\|Q\)\s*.*$/ contains=jiebaTestCaseDirectiveKeywords
syntax match jiebaTestCaseLocalDirective /^\(X\s\|M\s\|K\s\|O\s\|R\s\|C\s\|S0\s\|S1\s\|Q\s\|E\s\).*$/ contains=jiebaTestCaseDirectiveKeywords
syntax match jiebaTestCaseDirectiveKeywords /^\(#V\|#M\|#X\|#O\|#K\|#C\|#R\|#Q\|X\|M\|K\|O\|R\|C\|S0\|S1\|Q\|E\)/ contained

syntax match jiebaTestCaseHash /^H\s.*$/ contains=jiebaTestCaseHashKeyword
syntax match jiebaTestCaseHashKeyword /^H/ contained nextgroup=jiebaTestCaseHashId
syntax match jiebaTestCaseHashId /\v\s([a-f0-9]+|\?)(\s|$)/

syntax region jiebaTestCaseGlobalDirectiveBuffer start=/^#B/ end=/$/ contains=jiebaTestCaseBufferExpr
syntax region jiebaTestCaseLocalDirectiveBuffer start=/^B/ end=/$/ contains=jiebaTestCaseBufferExpr

" Buffer expressions
syntax match jiebaTestCaseBufferMark /\v[|\\<>\[\]]/ contained
syntax match jiebaTestCaseBufferEOL /␊/ contained
syntax match jiebaTestCaseBufferNul /␀/ contained
syntax match jiebaTestCaseBufferTab /┤/ contained
syntax match jiebaTestCaseBufferSpace /·/ contained
syntax match jiebaTestCaseBufferVisualFill /\~/ contained
syntax match jiebaTestCaseBufferBytesFill /@/ contained
syntax region jiebaTestCaseBufferExpr start=/^#\?B[01po]\s\+/ end=/$/ contains=jiebaTestCaseDirectiveBufferKeywords,jiebaTestCaseBufferMark,jiebaTestCaseBufferEOL,jiebaTestCaseBufferNul,jiebaTestCaseBufferTab,jiebaTestCaseBufferSpace,jiebaTestCaseBufferVisualFill,jiebaTestCaseBufferBytesFill containedin=jiebaTestCaseGlobalDirectiveBuffer,jiebaTestCaseLocalDirectiveBuffer
syntax match jiebaTestCaseDirectiveBufferKeywords /^\(#\?B0\|B1\|Bp\|Bo\)/ contained nextgroup=jiebaTestCaseBufferMark,jiebaTestCaseBufferEOL,jiebaTestCaseBufferNul,jiebaTestCaseBufferSpace,jiebaTestCaseBufferVisualFill,jiebaTestCaseBufferBytesFill


hi def link jiebaTestCaseComment Comment
hi def link jiebaTestCaseHeadConditional Keyword
hi def link jiebaTestCaseResetDefaults Type
hi def link jiebaTestCaseDirectiveKeywords Keyword
hi def link jiebaTestCaseHashKeyword Keyword
hi def link jiebaTestCaseHashId Comment
hi def link jiebaTestCaseDirectiveBufferKeywords Keyword
hi def link jiebaTestCaseBufferMark Number
hi def link jiebaTestCaseBufferEOL Comment
hi def link jiebaTestCaseBufferNul Comment
hi def link jiebaTestCaseBufferTab Comment
hi def link jiebaTestCaseBufferSpace Comment
hi def link jiebaTestCaseBufferVisualFill Comment
hi def link jiebaTestCaseBufferBytesFill Comment

let b:current_syntax = "jiebaTestCase_test_case"
