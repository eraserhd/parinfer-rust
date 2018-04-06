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

## E523 regression
    (foo bar
         baz)

After `wwciw(hi<Esc>j.`:
    (foo (hi)
         (hi))

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
