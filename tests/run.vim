set t_ti= t_te= background=dark nomore cpo-=C

if exists('$VIM_TO_TEST')
  let g:vim_to_test = $VIM_TO_TEST
else
  let g:vim_to_test = v:progname
endif

let s:features = {}
let s:current_feature = v:false
let s:current_scenario = v:false

function! s:gherkin(command, text)
  if a:command ==# "Feature"
    let s:current_feature = a:text
  elseif a:command ==# "Scenario"
    let s:current_scenario = a:text
  else
    call assert_true(s:current_feature, "Not currently in a Feature section")
    if !has_key(s:features, s:current_feature)
      let s:features[s:current_feature] = {}
    endif
    if !has_key(s:features[s:current_feature], s:current_scenario)
      let s:features[s:current_feature][s:current_scenario] = { "Give": [], "When": [], "Then": [] }
    endif
    let s:features[s:current_feature][s:current_scenario][a:command] += [ a:text[1:-1] ]
  endif
endfunction

command! -nargs=1 Feature call <SID>gherkin("Feature", <q-args>)
command! -nargs=1 Scenario call <SID>gherkin("Scenario", <q-args>)
command! -nargs=1 Give call <SID>gherkin("Give", <q-args>)
command! -nargs=1 When call <SID>gherkin("When", <q-args>)
command! -nargs=1 Then call <SID>gherkin("Then", <q-args>)

for testfile in glob("tests/test_*.vim", v:false, v:true)
  execute "source " . testfile
endfor

function s:run(scenario)
  let l:filename = tempname() . ".clj"
  call writefile(a:scenario["Give"], l:filename)
  let l:options = { "hidden": 1 }
  let l:term = term_start(g:vim_to_test . " --clean -n -u tests/vimrc " . l:filename, l:options)
  call term_setkill(l:term, "kill")
  sleep 1
  call term_wait(l:term, 1000)
  for l:key in split(join(a:scenario["When"], "<Enter>"), '\([^<]\|<[^>]*>\)\zs')
    if len(l:key) > 0 && l:key[0] ==# '<'
      let l:key = eval('"\' . l:key . '"')
    endif
    call term_sendkeys(l:term, l:key)
    call term_wait(l:term, 200)
  endfor

  for l:i in range(len(a:scenario["Then"]))
    let l:actual = substitute(term_getline(l:term, l:i+1), '\s\+$', '', '')
    if a:scenario["Then"][l:i] !=# l:actual
      let v:errors += [ "Line " . (l:i + 1) . " was '" . l:actual . "' not '" . a:scenario["Then"][l:i] . "'." ]
    endif
  endfor

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
        echo "    -" l:error
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
