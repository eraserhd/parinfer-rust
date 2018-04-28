# ChangeLog
## [Unreleased]

## [v0.2.0]
### Fixed
- `>>` reindents the rest of the form
- `E121: Undefined variable: w:parinfer_previous_cursor` after `:split`
- `E122` after `:PlugUpdate` (#18)
- `E523: not allowed here`
- `vim-fireplace` compatibility:
  - Fixed error after selecting expression with `cqq` (#15)

### Changed
- Honors `g:parinfer_enabled` (instead of `g:parinfer_mode` of "off")
- Is disabled during `:set paste`

### Added
- `:ParinferOn` command
- A logging facility and the `:ParinferLog` command
- `g:parinfer_force_balance` option (defaults to off)

### Removed
- `:ParinferToggleMode` (use `g:parinfer_mode` instead)

## 0.1.0
### Vim Plugin
#### Fixed
- `c` commands do not smart-dedent trailing lines

[Unreleased]: https://github.com/eraserhd/parinfer-rust/compare/v0.2.0...HEAD
[v0.2.0]: https://github.com/eraserhd/parinfer-rust/compare/v0.1.0...v0.2.0
