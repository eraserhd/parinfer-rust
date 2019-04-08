# parinfer-rust

Infer parentheses for Clojure, Lisp and Scheme.

https://github.com/eraserhd/parinfer-rust

A full-featured, super fast implementation of [Shaun Lebron's parinfer].  This
repo comes with Vim plugin files that work with Vim8 and Neovim.  The Rust
library can be called from other editors that can load dynamic libraries.

This plugin, unlike others available for Vim, implements "smart" mode.  Rather
than switching between "paren" mode and "indent" mode, parinfer uses
information about how the user is changing the file to decide what to do.

[Shaun Lebron's parinfer]: https://shaunlebron.github.io/parinfer/

## Installing

You need to have [rust installed](https://www.rust-lang.org/en-US/install.html).

### Vim and Neovim

#### `pathogen`

If you are using Tim Pope's `pathogen`:

    $ cd ~/.vim/bundle
    $ git clone git@github.com:eraserhd/parinfer-rust.git
    $ cd ~/.vim/bundle/parinfer-rust
    $ cargo build --release

#### `vim-plug`

```viml
Plug 'eraserhd/parinfer-rust'
```

Then, build project using cargo:

    $ cd /path/to/parinfer-rust
    $ cargo build --release

Or, with optional automatic recompilation on update:

```viml
Plug 'eraserhd/parinfer-rust', {'do':
        \  'cargo build --release'}
```

### Kakoune

#### `plug.kak`

Add this to your `kakrc`
```kak
plug "eraserhd/parinfer-rust" do %{
    cargo build --release
    cargo install --force --path .
}
```
Re-source your `kakrc` or restart Kakoune. Then run `:plug-install`. `plug.kak` will download, build and install plugin for you.

#### Manual

    $ cd ~/my-projects
    $ git clone git@github.com:eraserhd/parinfer-rust.git
    $ cd parinfer-rust
    $ make install
    $ cargo build --release
    $ cargo install

This links

## Building WebAssembly

WebAssembly currently needs the "nigthly" toolchain:

    $ rustup update
    $ rustup install nightly
    $ rustup target add wasm32-unknown-unknown --toolchain nightly
    $ cargo +nightly install cargo-web

It can then be built with:

    $ cargo +nightly web build --release

## Tests

You can run tests like so:

    $ cargo test                   # Run the native tests
    $ cargo +nightly web test      # Test the WebAssembly version
    $ vim --clean -u tests/run.vim # Integration tests

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
