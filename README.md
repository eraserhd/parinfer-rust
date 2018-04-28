# parinfer-rust

https://github.com/eraserhd/parinfer-rust

A full-featured, super snappy port of [Shaun Lebron's parinfer] to Rust.  This
repo comes with Vim plugin files that work with Vim8 and Neovim.  The Rust
library can be called from other editors that can load dynamic libraries.

This plugin, unlike others available for Vim, implements "smart" mode.  Rather
than switching between "paren" mode and "indent" mode, parinfer uses
information about how the user is changing the file to decide what to do.

[Shaun Lebron's parinfer]: https://shaunlebron.github.io/parinfer/

## Installing

You need to have [rust installed](https://www.rust-lang.org/en-US/install.html).

### `pathogen`

If you are using Tim Pope's `pathogen`:

    $ cd ~/.vim/bundle
    $ git clone git@github.com:eraserhd/parinfer-rust
    $ cd ~/.vim/bundle/parinfer-rust/cparinfer
    $ cargo build --release

### `vim-plug`

```viml
Plug 'eraserhd/parinfer-rust'
```

Then, build project using cargo:

    $ cd /path/to/parinfer-rust/cparinfer
    $ cargo build --release

Or, with optional automatic recompilation on update:

```viml
Plug 'eraserhd/parinfer-rust', {'do':
        \  'cargo build --manifest-path=cparinfer/Cargo.toml --release'}
```

## Building WebAssembly

    $ rustup update nightly
    $ cargo install cargo-web
    $ cargo web build

## Tests

You can run tests like so:

    $ vim --clean -u tests/run.vim

Tests are in a nice, readable format in `tests/test_*.vim`.  Please add tests
for any new features (or even old ones!).  You can set the `VIM_TO_TEST`
environment variable to Vim's path to test weird or different builds.

## Contributors

This wouldn't be possible without the work of others:

* Shaun Lebron - Inventing parinfer and doing the math.
* Case Nelson - Writing the nvim-parinfer, from which VimL code and some
  inspiration  was stolen.

## License

[ISC License](LICENSE.md)
