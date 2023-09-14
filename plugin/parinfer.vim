if !exists('g:parinfer_mode')
  let g:parinfer_mode = "smart"
endif
if !exists('g:parinfer_enabled')
  let g:parinfer_enabled = 1
endif
if !exists('g:parinfer_force_balance')
  let g:parinfer_force_balance = 0
endif
if !exists('g:parinfer_comment_char')
  let g:parinfer_comment_char = ";"
endif
if !exists('g:parinfer_string_delimiters')
  let g:parinfer_string_delimiters = ['"']
endif
if !exists('g:parinfer_lisp_vline_symbols')
  let g:parinfer_lisp_vline_symbols = 0
endif
if !exists('g:parinfer_lisp_block_comments')
  let g:parinfer_lisp_block_comments = 0
endif
if !exists('g:parinfer_guile_block_comments')
  let g:parinfer_guile_block_comments = 0
endif
if !exists('g:parinfer_scheme_sexp_comments')
  let g:parinfer_scheme_sexp_comments = 0
endif
if !exists('g:parinfer_janet_long_strings')
  let g:parinfer_janet_long_strings = 0
endif

" Needs to be outside function because we want <sfile> to be the location of this file,
" not where it is getting called from.
let s:libdir = expand('<sfile>:p:h:h') . '/target/release'

function! s:guess_dylib_path() abort
  if has('macunix')
    return s:libdir . '/libparinfer_rust.dylib'
  elseif has('unix')
    let s:uname = system("uname")
    if s:uname == "Darwin\n"
      return s:libdir . '/libparinfer_rust.dylib'
    else
      return s:libdir . '/libparinfer_rust.so'
    endif
  elseif has('win32')
    return s:libdir . '/parinfer_rust.dll'
  else
    " I hope we don't come here!
    echoerr 'Parinfer was unable to guess its dynamic library path.'
  endif
endfunction

command! ParinferOn let g:parinfer_enabled = 1
command! ParinferOff let g:parinfer_enabled = 0

" Common Lisp and Scheme: ignore parens in symbols enclosed by ||
au BufNewFile,BufRead *.lsp,*.lisp,*.cl,*.L,sbclrc,.sbclrc let b:parinfer_lisp_vline_symbols = 1
au BufNewFile,BufRead *.scm,*.sld,*.ss,*.rkt let b:parinfer_lisp_vline_symbols = 1

" Common Lisp and Scheme: ignore parens in block comments
au BufNewFile,BufRead *.lsp,*.lisp,*.cl,*.L,sbclrc,.sbclrc let b:parinfer_lisp_block_comments = 1
au BufNewFile,BufRead *.scm,*.sld,*.ss,*.rkt let b:parinfer_lisp_block_comments = 1

" Scheme (SRFI-62): S-expression comment
au BufNewFile,BufRead *.scm,*.sld,*.ss,*.rkt let b:parinfer_scheme_sexp_comments = 1

" Comment settings
au BufNewFile,BufRead *.janet let b:parinfer_comment_char = "#"

" Quote settings
au BufNewFile,BufRead *.yuck let b:parinfer_string_delimiters = ['"', "'", "`"]

" Long strings settings
au BufNewFile,BufRead *.janet let b:parinfer_janet_long_strings = 1

" Logging {{{1

function! s:parinfer_log(...)
  if a:0 > 0
    let g:parinfer_logfile = a:1
    echomsg 'Parinfer is now logging to '.a:1
  else
    unlet g:parinfer_logfile
    echomsg 'Parinfer is no longer logging'
  endif
endfunction

function! s:log(tag, data) abort
  if exists('g:parinfer_logfile')
    call writefile([printf('%20s: %s', a:tag, json_encode(a:data))], g:parinfer_logfile, 'a')
  endif
endfunction

function! s:log_diff(from, to) abort
  if exists('g:parinfer_logfile')
    let l:from_lines = split(a:from, "\n")
    let l:to_lines = split(a:to, "\n")

    " Compute the edit distance
    let l:table = map(range(0, len(l:from_lines)), 'repeat([19999], 1+len(l:to_lines))')
    for i in range(0, len(l:from_lines)) | let l:table[i][0] = i | endfor
    for j in range(0, len(l:to_lines)) | let l:table[0][j] = j | endfor
    for i in range(1, len(l:from_lines))
      for j in range(1, len(l:to_lines))
        let l:table[i][j] = min([ 1 + l:table[i-1][j], 1 + l:table[i][j-1] ])
        if l:from_lines[i-1] ==# l:to_lines[j-1]
          let l:table[i][j] = min([ l:table[i][j], 0 + l:table[i-1][j-1] ])
        endif
      endfor
    endfor

    " Construct a diff
    let l:i = len(l:from_lines)
    let l:j = len(l:to_lines)
    let l:diff = []
    while l:i > 0 || l:j > 0
      if l:i > 0 && l:j > 0 && l:table[i-1][j-1] == l:table[i][j] && l:from_lines[i-1] ==# l:to_lines[j-1]
        let l:diff += ['     ' . l:from_lines[i-1]]
        let l:i -= 1
        let l:j -= 1
      elseif l:j > 0 && 1+l:table[i][j-1] == l:table[i][j]
        let l:diff += ['    +' . l:to_lines[j-1]]
        let l:j -= 1
      elseif l:i > 0 && 1+l:table[i-1][j] == l:table[i][j]
        let l:diff += ['    -' . l:from_lines[i-1]]
        let l:i -= 1
      else
        throw 'bad case ' . l:i . ',' . l:j
      endif
    endwhile

    call writefile(reverse(l:diff), g:parinfer_logfile, 'a')
  endif
endfunction

command! -nargs=? ParinferLog call <SID>parinfer_log(<f-args>)

" Cursor Position {{{1

function! s:get_cursor_position() abort
  let l:cursor = getpos('.')
  let l:line = getline('.')
  let l:byte_column = l:cursor[2] - 1
  let l:cursor[1] -= 1
  let l:cursor[2] = strdisplaywidth(strpart(l:line, 0, l:byte_column))
  return l:cursor
endfunction

function! s:set_cursor_position(position) abort
  let l:cursor = copy(a:position)
  let l:cursor[1] += 1
  let l:cursor[2] += 1

  let line = getline(l:cursor[1])
  let head = matchstr(line, '.\+\%<' . (l:cursor[2] + 1) . 'v') " text before cursor
  let l:cursor[2] = strlen(head) + 1
  call setpos('.', l:cursor)
endfunction

" }}}

function! s:enter_window() abort
  let w:parinfer_previous_cursor = s:get_cursor_position()
endfunction

function! s:enter_buffer() abort
  call s:enter_window()
  if !exists('b:parinfer_last_changedtick')
    let b:parinfer_last_changedtick = -10
    let b:parinfer_previous_text = join(getline(1,'$'),"\n")
  endif
  let orig_mode = g:parinfer_mode
  let g:parinfer_mode = 'paren'
  call s:process_buffer()
  let g:parinfer_mode = orig_mode
endfunction

function! s:process_buffer() abort
  if !g:parinfer_enabled || &paste || !&modifiable
    return
  endif
  if !exists('b:parinfer_last_changedtick')
    call s:enter_buffer()
  endif
  if !exists('b:parinfer_comment_char')
    let b:parinfer_comment_char = g:parinfer_comment_char
  endif
  if !exists('b:parinfer_string_delimiters')
    let b:parinfer_string_delimiters = g:parinfer_string_delimiters
  endif
  if !exists('b:parinfer_lisp_vline_symbols')
    let b:parinfer_lisp_vline_symbols = g:parinfer_lisp_vline_symbols
  endif
  if !exists('b:parinfer_lisp_block_comments')
    let b:parinfer_lisp_block_comments = g:parinfer_lisp_block_comments
  endif
  if !exists('b:parinfer_guile_block_comments')
    let b:parinfer_guile_block_comments = g:parinfer_guile_block_comments
  endif
  if !exists('b:parinfer_scheme_sexp_comments')
    let b:parinfer_scheme_sexp_comments = g:parinfer_scheme_sexp_comments
  endif
  if !exists('b:parinfer_janet_long_strings')
    let b:parinfer_janet_long_strings = g:parinfer_janet_long_strings
  endif
  if b:parinfer_last_changedtick != b:changedtick
    let l:cursor = s:get_cursor_position()
    let l:orig_lines = getline(1,'$')
    let l:orig_text = join(l:orig_lines, "\n")
    let l:request = { "mode": g:parinfer_mode,
                    \ "text": l:orig_text,
                    \ "options": { "commentChar": b:parinfer_comment_char,
                                 \ "stringDelimiters": b:parinfer_string_delimiters,
                                 \ "cursorX": l:cursor[2],
                                 \ "cursorLine": l:cursor[1],
                                 \ "forceBalance": g:parinfer_force_balance ? v:true : v:false,
                                 \ "lispVlineSymbols": b:parinfer_lisp_vline_symbols ? v:true : v:false,
                                 \ "lispBlockComments": b:parinfer_lisp_block_comments ? v:true : v:false,
                                 \ "guileBlockComments": b:parinfer_guile_block_comments ? v:true : v:false,
                                 \ "schemeSexpComments": b:parinfer_scheme_sexp_comments ? v:true : v:false,
                                 \ "janetLongStrings": b:parinfer_janet_long_strings ? v:true : v:false,
                                 \ "prevCursorX": w:parinfer_previous_cursor[2],
                                 \ "prevCursorLine": w:parinfer_previous_cursor[1],
                                 \ "prevText": b:parinfer_previous_text } }
    if !exists('g:parinfer_dylib_path')
      let g:parinfer_dylib_path = s:guess_dylib_path()
    endif
    let l:response = json_decode(libcall(g:parinfer_dylib_path, "run_parinfer", json_encode(l:request)))
    if l:response["success"]
      if l:response["text"] !=# l:orig_text
        call s:log('change-request', l:request)
        call s:log('change-response', l:response)
        call s:log_diff(l:orig_text, l:response['text'])
        let l:lines = split(l:response["text"], "\n", 1)
        let l:changed = filter(range(len(l:lines)), 'l:lines[v:val] !=# l:orig_lines[v:val]')
        silent! undojoin
        try
          call setline(l:changed[0]+1, l:lines[l:changed[0]:l:changed[-1]])
        catch /E5\(23\|78\|65\):/ " not allowed here / not allowed to change text here / not allowed to chnage text or change window
          " If an event doesn't allow us to modify the buffer, that's OK.
          " Usually another event will happen before a redraw.
          call s:log('not-allowed-here', {})
        endtry
      endif
      let l:cursor[1] = l:response["cursorLine"]
      let l:cursor[2] = l:response["cursorX"]
      call s:set_cursor_position(l:cursor)

      let b:parinfer_previous_text = l:response["text"]
    else
      call s:log('error-response', l:response)
      let g:parinfer_last_error = l:response["error"]
      let b:parinfer_previous_text = join(getline(1,'$'),"\n")
    endif
    let b:parinfer_last_changedtick = b:changedtick
  endif
  let w:parinfer_previous_cursor = s:get_cursor_position()
endfunction

let s:EVENTS = {
  \ 'BufEnter': function('<SID>enter_buffer'),
  \ 'CursorMoved': function('<SID>process_buffer'),
  \ 'InsertCharPre': function('<SID>process_buffer'),
  \ 'InsertEnter': function('<SID>process_buffer'),
  \ 'TextChanged': function('<SID>process_buffer'),
  \ 'TextChangedI': function('<SID>process_buffer'),
  \ 'TextChangedP': function('<SID>process_buffer'),
  \ 'WinEnter': function('<SID>enter_window') }

function! s:event(name) abort
  call s:log('event', {'name': a:name, 'bufnr': bufnr('%'), 'changedtick': b:changedtick })
  call call(s:EVENTS[a:name], [])
endfunction

function! s:initialize_buffer() abort
  " We can't get the buffer in the command-line window, so don't initialize
  " it.  This happens with vim-fireplace's `cqq`.
  if getcmdwintype() !=# ''
    return
  endif

  " Don't enable if preview window
  if &previewwindow
    return
  endif

  for event_name in filter(keys(s:EVENTS),'exists("##".v:val)')
    execute "autocmd! Parinfer ".event_name." <buffer> call <SID>event('".event_name."')"
  endfor
endfunction

augroup Parinfer
  autocmd FileType clojure,scheme,lisp,racket,hy,fennel,janet,carp,wast,yuck,dune call <SID>initialize_buffer()
augroup END

" Handle the case where parinfer was lazy-loaded
if (&filetype ==? 'clojure' || &filetype ==? 'scheme' || &filetype ==? 'lisp' || &filetype ==? 'racket' || &filetype ==? 'hy' || &filetype ==? 'fennel' || &filetype ==? 'janet' || &filetype ==? 'carp' || &filetype ==? 'wast' || &filetype ==? 'yuck' || &filetype ==? 'dune')
  call <SID>initialize_buffer()
endif

let g:parinfer_loaded = v:true

" vim:set sts=2 sw=2 ai et foldmethod=marker:
