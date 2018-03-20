if !exists('g:parinfer_mode')
  let g:parinfer_mode = "indent"
endif

if !exists('g:parinfer_dylib_path')
	if has('macunix')
		let g:parinfer_dylib_path = expand('<sfile>:p:h:h'). '/cparinfer/target/release/libcparinfer.dylib'
	elseif has('unix')
		let g:parinfer_dylib_path = expand('<sfile>:p:h:h'). '/cparinfer/target/release/libcparinfer.so'
	elseif has('win32')
		let g:parinfer_dylib_path = expand('<sfile>:p:h:h'). '/cparinfer/target/release/cparinfer.dll'
	else
		" I hope we don't come here!
	endif
endif

function! s:toggleMode()
  if g:parinfer_mode == "indent"
    let g:parinfer_mode = "paren"
  else
    let g:parinfer_mode = "indent"
  endif
endfunction

function! s:turnOff()
  let g:parinfer_mode = "off"
endfunction

command! ParinferToggleMode call <SID>toggleMode()
command! ParinferOff call <SID>turnOff()

function! s:process(mode)
  if g:parinfer_mode != "off"
    let l:pos = getpos(".")
    let l:orig_lines = getline(1,line('$'))
    let l:orig_text = join(l:orig_lines, "\n")
    let l:request = { "mode": a:mode,
                    \ "text": l:orig_text,
                    \ "options": { "cursorX": l:pos[2] - 1,
                                 \ "cursorLine": l:pos[1] - 1 } }
    let l:response = json_decode(libcall(g:parinfer_dylib_path, "run_parinfer", json_encode(l:request)))
    if l:response["success"] 
      if l:response["text"] !=# l:orig_text
        let l:lines = split(l:response["text"], "\n", 1)
        let l:changed = filter(range(len(l:lines)), 'l:lines[v:val] !=# l:orig_lines[v:val]')
        
        try
          silent undojoin
        catch
        endtry
        call setline(l:changed[0]+1, l:lines[l:changed[0]:l:changed[-1]])
      endif
      let l:pos[1] = l:response["cursorLine"] + 1
      let l:pos[2] = l:response["cursorX"] + 1
      call setpos('.', l:pos)
    else
      let g:parinfer_last_error = l:result["error"]
    endif
  endif
endfunction

augroup Parinfer
  autocmd FileType clojure,scheme,lisp,racket,hy
        \ :autocmd! Parinfer BufEnter <buffer>
        \ :call <SID>process("paren")
  autocmd FileType clojure,scheme,lisp,racket,hy
        \ :autocmd! Parinfer TextChanged <buffer>
        \ :call <SID>process(g:parinfer_mode)
  autocmd FileType clojure,scheme,lisp,racket,hy
        \ :autocmd! Parinfer TextChangedI <buffer>
        \ :call <SID>process(g:parinfer_mode)
augroup END

" vim:set sts=2 sw=2 ai et:
