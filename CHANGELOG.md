# ChangeLog
## [Unreleased]
### Fixed
- `>>` reindents the rest of the form
- Honor `g:parinfer_enabled` (instead of `g:parinfer_mode` of "off")
- `E121: Undefined variable: w:parinfer_previous_cursor` after `:split`
- `E122` after `:PlugUpdate` (#18)
- `E523: not allowed here`
- `vim-fireplace` compatibility:
  - Fixed error after selecting expression with `cqq` (#15)

## 0.1.0
### Vim Plugin
#### Fixed
- `c` commands do not smart-dedent trailing lines

[Unreleased]: https://github.com/eraserhd/parinfer-rust/compare/v0.1.0...HEAD
