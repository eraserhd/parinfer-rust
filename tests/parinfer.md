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

## [Not Working] Undo (#14)

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
