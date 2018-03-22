set t_ti= t_te= background=dark nomore cpo-=C

function s:run_tests()
  redir @t
  silent function /^Test_
  redir END

  let l:ok = v:true
  echohl LineNr
  echo "Running tests:"
  echohl None
  echo "\n"
  for test in split(substitute(@t, 'function \(\w\+\)()', '\1', 'g'), "\n")
    let v:errors = []
    try
      silent call call(test, [])
    catch
      let v:errors += [v:throwpoint . ": ". v:exception]
    endtry
    echo "  " . test . "... "
    if len(v:errors) > 0
      let l:ok = v:false
      echohl ErrorMsg
      echon "failed"
      echohl None
      for error in v:errors
        echo "    - " . error
      endfor
    else
      echohl MoreMsg
      echon "ok"
      echohl None
    endif
  endfor
  echo "\n"

  if l:ok
    quit
  else 
    cquit
  endif
endfunction

filetype on

try
  source plugin/parinfer.vim
catch
  echohl ErrorMsg
  echo "Error loading Vim plugin:" v:exception
  echohl None
  cquit
endtry

for testfile in glob("tests/test_*.vim", v:false, v:true)
  execute "source " . testfile
endfor

call <SID>run_tests()
