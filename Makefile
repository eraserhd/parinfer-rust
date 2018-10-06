.PHONY: install
install:
	cargo build --release
	cargo install --force
	mkdir -p ~/.config/kak/autoload/
	for file in "$$(pwd)"/rc/*.kak; do \
	  ln -sf "$${file}" ~/.config/kak/autoload/; \
	done

