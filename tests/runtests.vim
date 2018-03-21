set t_ti= t_te= background=dark nomore

function s:exec(step_name, working_directory, shell_command)
  echohl LineNr
  echo a:step_name . ":\n\n"
  echohl None
  let l:start_directory = getcwd()
  execute "chdir " . a:working_directory
  silent execute "!" . a:shell_command
  if v:shell_error
    cquit
  endif
  execute "chdir " . l:start_directory
  echo "\n"
endfunction

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
    echo "  " . test . "... "
    call call(test, [])
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

  return l:ok
endfunction

call s:exec('Testing parinfer', 'parinfer', 'cargo test')
call s:exec('Testing cparinfer', 'cparinfer', 'cargo test')
call s:exec('Building release plugin', 'cparinfer', 'cargo build --release')
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

if s:run_tests()
  quit
else
  cquit
endif
