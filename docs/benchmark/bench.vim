function! Bench(bang, measure_warmup)
    let l:mycount = a:measure_warmup ? 10 : 50000
    let l:output = []
    let l:cmd = a:bang ? "normal! w" : "normal w"
    while l:mycount > 0
        let l:mycount -= 1
        let l:start = reltime()
        execute l:cmd
        let l:tm = reltimestr(reltime(l:start))
        call add(l:output, l:tm)
    endwhile
    let l:label = a:bang ? "std" : "custom"
    call writefile(l:output, "bench_output_" . l:label . ".txt")
endfunction

let s:measure_warmup = exists("$MEASURE_WARMUP") ? 1 : 0
let s:std_run = exists("$RUN_BUILTIN") ? 1 : 0

if !s:std_run && !exists("g:loaded_jieba_vim")
    cquit
    finish
endif
call Bench(s:std_run, s:measure_warmup)
quit
