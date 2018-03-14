# parinfer-rust

A port of Shaun Lebron's parinfer to Rust.  Why?  To make a snappy plugin that
can be called with Vim's libcall(), saving wire transfer time, and working on
Vim 8 and not just Neovim.

## Installing

You need to have the rust compiler and `cargo` installed.

If you are using Tim Pope's `pathogen`:

    $ cd ~/.vim/bundle
    $ git clone git@github.com:eraserhd/parinfer-rust
    $ cd ~/.vim/bundle/parinfer-rust/cparinfer
    $ cargo build --release

## Contributors

This wouldn't be possible without the work of others:

* Shaun Lebron - Inventing parinfer and doing the math.
* Case Nelson - Writing the nvim-parinfer, from which VimL code and some
  inspiration  was stolen.
