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
