set t_ti= t_te= background=dark nomore cpo-=C

if exists('$VIM_TO_TEST')
  let g:vim_to_test = $VIM_TO_TEST
else
  let g:vim_to_test = v:progname
endif

let s:features = {}
let s:current_feature = v:false
let s:current_scenario = v:false

function! s:add_scenario_part(command, text)
  call assert_true(s:current_feature, "Not currently in a Feature section")
  if !has_key(s:features, s:current_feature)
    let s:features[s:current_feature] = {}
  endif
  if !has_key(s:features[s:current_feature], s:current_scenario)
    let s:features[s:current_feature][s:current_scenario] = { "Give": [], "When": [], "Then": [] }
  endif
  let s:features[s:current_feature][s:current_scenario][a:command] += [ a:text ]
endfunction

function! s:load_feature(filename)
  let l:state = 'before_give'
  for l:line in readfile(a:filename)
    if l:line =~ '^# '
      let s:current_feature = substitute(l:line, '^# ', '', '')
    elseif l:line =~ '^## '
      let s:current_scenario = substitute(l:line, '^## ', '', '')
      let l:state = 'before_give'
    elseif l:line =~ '^\s*$' && l:state ==# 'in_give'
      let l:state = 'in_then'
    elseif l:line =~ '^    ' && (l:state ==# 'before_give' || l:state ==# 'in_give')
      call s:add_scenario_part("Give", substitute(l:line, '^    ', '', ''))
      let l:state = 'in_give'
    elseif l:line =~ '`.\+`' && (l:state ==# 'in_then' || l:state ==# 'in_give')
      call s:add_scenario_part("When", substitute(l:line, '^.*`\(.\+\)`.*$', '\1', ''))
      let l:state = 'in_then'
    elseif l:line =~ '^    ' && l:state ==# 'in_then'
      call s:add_scenario_part("Then", substitute(l:line, '^    ', '', ''))
    endif
  endfor
endfunction

for feature in glob("tests/*.md", v:false, v:true)
  call s:load_feature(feature)
endfor

function s:run(scenario)
  let l:filename = tempname() . ".clj"
  call writefile(a:scenario["Give"], l:filename)
  let l:options = {
    \ 'hidden': 1,
    \ 'term_rows': 15,
    \ 'term_cols': 30,
    \ 'term_kill': 'kill' }
  let l:term = term_start(g:vim_to_test . " -n -u tests/vimrc " . l:filename, l:options)
  sleep 2
  call term_wait(l:term, 1000)
  for l:key in split(join(a:scenario["When"], "<Enter>"), '\([^<]\|<[^>]*>\)\zs')
    if len(l:key) > 0 && l:key[0] ==# '<'
      let l:key = eval('"\' . l:key . '"')
    endif
    call term_sendkeys(l:term, l:key)
    call term_wait(l:term, 200)
  endfor
  call term_sendkeys(l:term, ":w\<CR>")
  call term_wait(l:term, 200)
  call job_stop(term_getjob(l:term))

  let l:actual = readfile(l:filename)
  if a:scenario["Then"] !=# l:actual
    let v:errors += [ "Expected:\n" . join(a:scenario["Then"],"\n") . "\nActual:\n" . join(l:actual,"\n") ]
  endif

  execute "bdelete! " . l:term
  call delete(l:filename)
endfunction

function s:run_all()
  let l:ok = v:true
  for l:feature_name in keys(s:features)
    echohl LineNr
    echo l:feature_name
    echohl None
    for l:scenario_name in keys(s:features[l:feature_name])
      let v:errors = []
      call s:run(s:features[l:feature_name][l:scenario_name])
      if len(v:errors) > 0
        let l:ok = v:false
        echohl ErrorMsg
      else
        echohl MoreMsg
      endif
      echo "  -" l:scenario_name
      echohl None
      for l:error in v:errors
        echo l:error
      endfor
    endfor
  endfor
  if l:ok
    quit
  else
    cquit
  endif
endfunction

try
  source plugin/parinfer.vim
catch
  echohl ErrorMsg
  echo "Error loading Vim plugin:" v:exception
  echohl None
  cquit
endtry

call <SID>run_all()
