.PHONY: all clean

all:
	cargo install \
		--quiet \
		--path . \
		--root . \
		--no-track \
		--profile release \
		--target wasm32-unknown-unknown

clean:
	rm -rf bin
