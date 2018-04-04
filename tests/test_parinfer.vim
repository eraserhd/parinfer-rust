Feature parinfer
  Scenario fixes parens on initial load
    Give .(foo [
    Give . x])
    Then .(foo [
    Then .      x])

  Scenario smart mode pushes brackets
    Give .(foo [
    Give .      x])
    When .llib<Esc>
    Then .(fboo [
    Then .       x])

  Scenario change commands fix indent
    Give .(foo [
    Give .      x])
    When .lcwa<Esc>
    Then .(a [
    Then .    x])

  Scenario >> reindents rest of form
    Give .(foo
    Give .  (bar
    Give .    baz
    Give .    quux))
    When .j>>
    Then .(foo
    Then .    (bar
    Then .      baz
    Then .      quux))

  Scenario appending after (let [
    Give .(let [
    Give .      x])
    When .Ay<Esc>
    Then .(let [y
    Then .      x])

  Scenario E523 regression
    Give .(foo bar
    Give .     baz)
    When .wwciw(hi<Esc>j.
    Then .(foo (hi)
    Then .     (hi))
