# parinfer
## fixes parens on initial load

    (foo [
     x])

    (foo [
          x])

## smart mode pushes brackets

    (foo [
          x])

After `llib<Esc>`:
    (fboo [
           x])

## change commands fix indent

    (foo [
          x])

`lcwa<Esc>` will produce:
    (a [
        x])

## >> reindents rest of form

    (foo
      (bar
        baz
        quux))

After `j>>`:
    (foo
        (bar
          baz
          quux))

## appending after (let [

    (let [
          x])

After `Ay<Esc>`:
    (let [y
          x])

# Regressions
## E523
    (foo bar
         baz)

After `wwciw(hi<Esc>j.`:
    (foo (hi)
         (hi))

## E121

Splitting a window doesn't trigger BufEnter, because we don't switch buffers,
so we need to copy window-local state to the new window.

    (foo)

After `lli <Esc>:w<CR>:split<CR>lay<Esc>`
    (f oyo)

# Joining lines
## `J` joins two lines correctly (#10)

Joining lines with `J` should reindent the second line so as to not break
their meaning.

    (foo
      (bar
        baz))

After `J`:
    (foo (bar
           baz))

## `J` joins multiple lines correctly (#10)

    (foo
      (bar
        [baz
         quux]))

After `3J`:
    (foo (bar [baz
               quux]))

# Compatibility with vim-fireplace
## Works in an input command-line window (#15)

This test is very hackish.

    (foo)

After `:call setline(1,input('=>'))<CR><C-F>i(x<CR>`:

    (x)
