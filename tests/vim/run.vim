set t_ti= t_te= background=dark nomore cpo-=C

if exists('$VIM_TO_TEST')
  let g:vim_to_test = $VIM_TO_TEST
else
  let g:vim_to_test = v:progname
endif
if exists('$PLUGIN_TO_TEST')
  set runtimepath+=$PLUGIN_TO_TEST
else
  set runtimepath+=.
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
  let offset = 0
  let text = join(readfile(a:filename),"\n")."\n"
  let state = 0
  while offset < len(text)
    " Feature
    let mend = matchlist(text, '^#\s\+\([^\n]*\)\n', offset)
    if mend != []
      let s:current_feature = mend[1]
      let s:current_scenario = v:null
      if !has_key(s:features, s:current_feature)
        let s:features[s:current_feature] = {}
      endif
      let offset += len(mend[0])
      continue
    endif

    " Scenario (possibly tagged as disabled)
    let mend = matchlist(text, '^##\s\+\(\[[^\]]*\]\s*\|\)\([^\n]*\)\n', offset)
    if mend != []
      let s:current_scenario = mend[1] . mend[2]
      if !has_key(s:features[s:current_feature], s:current_scenario)
        let s:features[s:current_feature][s:current_scenario] = { "Give": [], "When": [], "Then": [] }
      endif
      if len(mend[1]) > 0
        let s:features[s:current_feature][s:current_scenario]['disabled'] = 1
      endif
      let offset += len(mend[0])
      let state = 'Give'
      continue
    endif

    " Code block
    let mend = matchlist(text, '^```\s*\n\(.\{-}\n\)```\s*\n', offset)
    if mend != []
      let s:features[s:current_feature][s:current_scenario][state] = split(mend[1], '\n')
      let state = 'Then'
      let offset += len(mend[0])
      continue
    endif

    " Commentary with Vim keys
    let mend = matchlist(text, '^[^\n`]*`\([^\n`]*\)`[^\n]*\n', offset)
    if mend != []
      if state ==# 'Then'
        let s:features[s:current_feature][s:current_scenario]['When'] += [mend[1]]
      endif
      let offset += len(mend[0])
      continue
    endif

    " Commentary with no keys
    let mend = matchlist(text, '^[^\n]*\n', offset)
    if mend != []
      let offset += len(mend[0])
      continue
    endif
  endwhile
endfunction

for feature in glob(expand('<sfile>:p:h') . "/*.md", v:false, v:true)
  call s:load_feature(feature)
endfor

let s:test_vimrc_path = expand('<sfile>:p:h') . "/vimrc"

function s:run(scenario) abort
  if has_key(a:scenario, 'disabled')
    return
  endif
  let l:filename = tempname() . ".clj"
  call writefile(a:scenario["Give"], l:filename)
  let l:options = {
    \ 'hidden': 1,
    \ 'term_rows': 15,
    \ 'term_cols': 30,
    \ 'term_kill': 'kill' }
  let l:term = term_start(g:vim_to_test . " -n -u " . s:test_vimrc_path . " " . l:filename, l:options)
  sleep 2
  call term_wait(l:term, 1000)
  for l:key in split(join(a:scenario["When"], "<Enter>"), '\([^<]\|<[^>]*>\)\zs')
    call writefile([printf('%20s: "%s"', 'keys', l:key)], l:filename . '.log', 'a')
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
    let v:errors += [
      \ "Expected:\n" .
      \ join(a:scenario["Then"],"\n") . "\n" .
      \ "Actual:\n" .
      \ join(l:actual,"\n") . "\n" .
      \ "Log:\n" .
      \ join(readfile(l:filename . '.log'),"\n") . "\n" ]
  endif

  execute "bdelete! " . l:term
  call delete(l:filename)
  call delete(l:filename . '.log')
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
      elseif has_key(s:features[l:feature_name][l:scenario_name], 'disabled')
        echohl PmenuSel
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
  runtime plugin/parinfer.vim
catch
  echohl ErrorMsg
  echo "Error loading Vim plugin:" v:exception
  echohl None
  cquit
endtry

call <SID>run_all()
