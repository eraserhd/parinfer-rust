set nomore

function s:exec(working_directory, shell_command)
  let l:start_directory = getcwd()
  execute "chdir " . a:working_directory
  execute "silent !" . a:shell_command
  if v:shell_error
    cquit
  endif
  execute "chdir " . l:start_directory
endfunction

call s:exec('parinfer', 'cargo test')
call s:exec('cparinfer', 'cargo test')
call s:exec('cparinfer', 'cargo build --release')
source plugin/parinfer.vim

quit
