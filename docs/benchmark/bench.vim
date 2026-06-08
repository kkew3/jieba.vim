function! Bench(bang)
    let l:mycount = 50000
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

call Bench(0)
quit
