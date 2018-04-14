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
  let g:parinfer_enabled = 0
endfunction

command! ParinferToggleMode call <SID>toggleMode()
command! ParinferOff call <SID>turnOff()

function! s:bufEnter()
  let w:parinfer_previous_cursor = getpos(".")
  let b:parinfer_last_changedtick = -10
  let b:parinfer_previous_text = join(getline(1,line('$')),"\n")
  let orig_mode = g:parinfer_mode
  let g:parinfer_mode = 'paren'
  call s:process()
  let g:parinfer_mode = orig_mode
endfunction

function! s:process() abort
  if !g:parinfer_enabled
    return
  endif
  if b:parinfer_last_changedtick != b:changedtick
    let l:pos = getpos(".")
    let l:orig_lines = getline(1,line('$'))
    let l:orig_text = join(l:orig_lines, "\n")
    let l:request = { "mode": g:parinfer_mode,
                    \ "text": l:orig_text,
                    \ "options": { "cursorX": l:pos[2] - 1,
                                 \ "cursorLine": l:pos[1] - 1,
                                 \ "prevCursorX": w:parinfer_previous_cursor[2] - 1,
                                 \ "prevCursorLine": w:parinfer_previous_cursor[1] - 1,
                                 \ "prevText": b:parinfer_previous_text } }
    let l:response = json_decode(libcall(g:parinfer_dylib_path, "run_parinfer", json_encode(l:request)))
    if l:response["success"]
      if l:response["text"] !=# l:orig_text
        let l:lines = split(l:response["text"], "\n", 1)
        let l:changed = filter(range(len(l:lines)), 'l:lines[v:val] !=# l:orig_lines[v:val]')
        silent! undojoin
        try
          call setline(l:changed[0]+1, l:lines[l:changed[0]:l:changed[-1]])
        catch /E523:/ " not allowed here
          " If an event doesn't allow us to modify the buffer, that's OK.
          " Usually another event will happen before a redraw.
        endtry
      endif
      let l:pos[1] = l:response["cursorLine"] + 1
      let l:pos[2] = l:response["cursorX"] + 1
      call setpos('.', l:pos)

      let b:parinfer_previous_text = l:response["text"]
    else
      let g:parinfer_last_error = l:response["error"]
      let b:parinfer_previous_text = join(getline(1,line('$')),"\n")
    endif
    let b:parinfer_last_changedtick = b:changedtick
  endif
  let w:parinfer_previous_cursor = getpos(".")
endfunction

function! s:initialize_buffer() abort
  " We can't get the buffer in the command-line window, so don't initialize
  " it.  This happens with vim-fireplace's `cqq`.
  if getcmdwintype() !=# ''
    return
  endif
  autocmd! Parinfer BufEnter <buffer> call <SID>bufEnter()
  autocmd! Parinfer TextChanged <buffer> call <SID>process()
  autocmd! Parinfer InsertEnter <buffer> call <SID>process()
  autocmd! Parinfer InsertCharPre <buffer> call <SID>process()
  autocmd! Parinfer TextChangedI <buffer> call <SID>process()
  if exists('##TextChangedP')
    autocmd! Parinfer TextChangedP <buffer> call <SID>process()
  endif
  autocmd! Parinfer CursorMoved <buffer> call <SID>process()
endfunction

augroup Parinfer
  autocmd FileType clojure,scheme,lisp,racket,hy call <SID>initialize_buffer()
augroup END

" vim:set sts=2 sw=2 ai et:
