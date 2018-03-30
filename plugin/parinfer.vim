if !exists('g:parinfer_mode')
  let g:parinfer_mode = "smart"
endif
if !exists('g:parinfer_enabled')
  let g:parinfer_enabled = 1
endif

if !exists('g:parinfer_dylib_path')
  if has('macunix')
    let g:parinfer_dylib_path = expand('<sfile>:p:h:h'). '/cparinfer/target/release/libcparinfer.dylib'
  elseif has('unix')
    let s:uname = system("uname")
    if s:uname == "Darwin\n"
      let g:parinfer_dylib_path = expand('<sfile>:p:h:h'). '/cparinfer/target/release/libcparinfer.dylib'
    else
      let g:parinfer_dylib_path = expand('<sfile>:p:h:h'). '/cparinfer/target/release/libcparinfer.so'
    endif
  elseif has('win32')
    let g:parinfer_dylib_path = expand('<sfile>:p:h:h'). '/cparinfer/target/release/cparinfer.dll'
  else
    " I hope we don't come here!
  endif
endif

function! s:toggleMode()
  if g:parinfer_mode == "smart"
    let g:parinfer_mode = "indent"
  elseif g:parinfer_mode == "indent"
    let g:parinfer_mode = "paren"
  else
    let g:parinfer_mode = "smart"
  endif
endfunction

function! s:turnOff()
  let g:parinfer_mode = "off"
endfunction

command! ParinferToggleMode call <SID>toggleMode()
command! ParinferOff call <SID>turnOff()

function! s:saveCursorPos()
  let s:prevCursor = getpos(".")
  let s:prevText = join(getline(1,line('$')),"\n")
endfunction

function! s:bufEnter()
  call s:saveCursorPos()
  call s:process('paren')
endfunction

function! s:insertEnter()
  if v:operator ==# 'c'
    let v:char = 'x'
    call s:process('smart')
  endif
endfunction

function! s:process(mode)
  if g:parinfer_mode == "off"
    return
  endif
  let l:pos = getpos(".")
  let l:orig_lines = getline(1,line('$'))
  let l:orig_text = join(l:orig_lines, "\n")
  let l:request = { "mode": a:mode,
                  \ "text": l:orig_text,
                  \ "options": { "cursorX": l:pos[2] - 1,
                               \ "cursorLine": l:pos[1] - 1,
                               \ "prevCursorX": s:prevCursor[2] - 1,
                               \ "prevCursorLine": s:prevCursor[1] - 1,
                               \ "prevText": s:prevText } }
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
    call s:saveCursorPos()
  else
    let g:parinfer_last_error = l:response["error"]
  endif
endfunction

function! s:initialize_buffer()
  autocmd! Parinfer BufEnter <buffer> call <SID>bufEnter()
  autocmd! Parinfer TextChanged <buffer> call <SID>process(g:parinfer_mode)
  autocmd! Parinfer InsertEnter <buffer> call <SID>insertEnter()
  autocmd! Parinfer InsertCharPre <buffer> call <SID>saveCursorPos()
  autocmd! Parinfer TextChangedI <buffer> call <SID>process(g:parinfer_mode)
  if exists('##TextChangedP')
    autocmd! Parinfer TextChangedP <buffer> call <SID>process(g:parinfer_mode)
  endif
  autocmd! Parinfer CursorMoved <buffer> call <SID>saveCursorPos()
  call <SID>saveCursorPos()
endfunction

augroup Parinfer
  autocmd FileType clojure,scheme,lisp,racket,hy call <SID>initialize_buffer()
augroup END

" vim:set sts=2 sw=2 ai et:
