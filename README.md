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

If you are using `vim-plug`:

	Plug 'eraserhd/parinfer-rust'

Then, build project using cargo:

	$ cd /path/to/parinfer-rust/cparinfer
	$ cargo build --release

Or, with optional automatic recompilation on update:

	Plug 'eraserhd/parinfer-rust', {'do':
		\  'cargo build --manifest-path=cparinfer/Cargo.toml --release'}

## Tests

You can run tests like so:

    $ vim --clean -u tests/run.vim

Tests are in a nice, readable format in `tests/test_*.vim`.  Please add tests
for any new features (or even old ones!).

## Contributors

This wouldn't be possible without the work of others:

* Shaun Lebron - Inventing parinfer and doing the math.
* Case Nelson - Writing the nvim-parinfer, from which VimL code and some
  inspiration  was stolen.

## License

[ISC License](LICENSE.md)
