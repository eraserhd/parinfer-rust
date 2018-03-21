
function! Test_open_runs_paren_mode_first()
  edit tests/basic.clj
  call assert_equal("clojure", &ft)
  call assert_equal("(foo [\n      x])", join(getline(1,'$'), "\n"))
  enew!
endfunction

function! Test_bar()
  call assert_equal(2,2)
endfunction
