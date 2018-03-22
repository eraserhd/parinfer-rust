if exists('$VIM_TO_TEST')
  let g:vim_to_test = $VIM_TO_TEST
else
  let g:vim_to_test = v:progname
endif


function! Test_open_fixes_broken_parens()
  edit tests/basic.clj
  call assert_equal("clojure", &ft)
  call assert_equal("(foo [\n      x])", join(getline(1,'$'), "\n"))
  enew!
endfunction

function! Test_smart_mode_works()
  let l:options = { "hidden": 1 }
  let l:term = term_start(g:vim_to_test . " --clean -n -u tests/vimrc tests/basic.clj", l:options)
  call term_setkill(l:term, "kill")
  sleep 1
  call term_wait(l:term,1000)
  call term_sendkeys(l:term, "lli")
  call term_wait(l:term,1000)
  call term_sendkeys(l:term, "b")
  call term_wait(l:term)
  call term_sendkeys(l:term, "\<Esc>")
  call term_wait(l:term)
  call assert_equal("(fboo [", term_getline(l:term,1))
  call assert_equal("       x])", term_getline(l:term,2))
  execute "bdelete! " . l:term
endfunction
