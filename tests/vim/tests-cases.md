# Parinfer
## Fixes parens on initial load

```
(foo [
 x])
```

```
(foo [
      x])
```

## Smart mode pushes brackets

```
(foo [
      x])
```

After `llib<Esc>`:
```
(fboo [
       x])
```

## [Not Working] "force balance" mode can be enabled

This is currently not working due to an issue where the Rust port does not
match the JavaScript implementation when "forceBalance" is set.

```
(foo [:baz])
```

After `:set lisp<CR>:let g:parinfer_force_balance=1<CR>f]i<Enter><Esc>`:
```
(foo [:baz])

```

## Is disabled when `&paste` is set

Vim's `&paste` option is for pasting into the terminal.  Vim receives the
text as many keystrokes, essentially, so we should not be reformatting the
code as it is being added.

```
(foo)
```

After `:set paste<CR>i))<Esc>`:
```
))(foo)
```

## change commands fix indent

```
(foo [
      x])
```

`lcwa<Esc>` will produce:

```
(a [
    x])
```

## >> reindents rest of form

```
(foo
  (bar
    baz
    quux))
```

After `j>>`:

```
(foo
    (bar
      baz
      quux))
```

## appending after (let [

```
(let [
      x])
```

After `Ay<Esc>`:

```
(let [y
      x])
```

# Regressions
## E523

```
(foo bar
     baz)
```

After `wwciw(hi<Esc>j.`:
```
(foo (hi)
     (hi))
```

## E121

Splitting a window doesn't trigger BufEnter, because we don't switch buffers,
so we need to copy window-local state to the new window.

```
(foo)
```

After `lli <Esc>:w<CR>:split<CR>lay<Esc>`
```
(f oyo)
```

## Undefined variable `b:parinfer_last_changedtick` (#27, #28)

```
(foo)
```

After `:unlet b:parinfer_last_changetick b:parinfer_previous_text w:parinfer_previous_cursor<CR>ixx<Esc>`:

```
xx(foo)
```

## [Not Working] Deleting to end-of-line (#21)

Normal-mode commands which delete to the end of the line move the cursor back
one space to keep it within the line.  This confuses parinfer when we don't
accomodate this.

```
(let [a "foo"])
```

After `f"D`:
```
(let [a ])
```

# Joining lines
## `J` joins two lines correctly (#10)

Joining lines with `J` should reindent the second line so as to not break
their meaning.

```
(foo
  (bar
    baz))
```

After `J`:
```
(foo (bar
       baz))
```

## `J` joins multiple lines correctly (#10)

```
(foo
  (bar
    [baz
     quux]))
```

After `3J`:
```
(foo (bar [baz
           quux]))
```

# Compatibility with vim-fireplace
## Works in an input command-line window (#15)

This test is very hackish.

```
(foo)
```

After `:call setline(1,input('=>'))<CR><C-F>i(x<CR>`:
```
(x)
```

# Undo (#14)
## Regular Vim undo

This case shouldn't need to do anything fancy to mess with Vim's undo
handling.

```
x
y
```

After `2GO<Esc>O<Esc>u`:
```
x

y
```

## Undo after triggering parinfer in insert mode

This triggers a parinfer update while in insert mode.  In theory, we shouldn't
need to call "undojoin" here because Vim groups all insert changes (and our
call to |setline()|).

```
x
y
```

After `2GO<Esc>O[<Esc>u`:
```
x

y
```

## Undo after triggering parinfer (closing) in insert mode

This one is like the last, except that it will delete an extra closing brace,
which will hopefully not mess with Vim.

```
x
y
```

After `2GO<Esc>O[]<Esc>u`:
```
x

y
```

## Undo with multiple parinfer changes in insert (#14)

This requires Vim >= 8.1.0256.

```
(defn f
  [x]
  x)

(defn g
  [y]
  y)
```

After `4GO<Enter>(defn h<Enter>  [z]<Enter>z)<Esc>u`:
```
(defn f
  [x]
  x)

(defn g
  [y]
  y)
```

# Unicode (#26)
## hukka's example (#26)

```
(def aaa {:foo 1
          :bar 1})
```

After `wwciwäää<Esc>:`
```
(def äää {:foo 1
          :bar 1})
```

## Smart mode after composed characters (#26)

Vim's getpos('.') returns the column in byte count, so we need to use
virtcol('.') to give a good column number to parinfer-rust.

```
(def å [])
```

After `f]ix<Esc>`:
```
(def å [x])
```

## Cursor position with fullwidth text (#87)

```
;; １２３４５
```

After `6lixy<Esc>`:
```
;; １２３xy４５
```
