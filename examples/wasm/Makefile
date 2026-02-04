.PHONY: all clean

all:
	cargo install \
		--quiet \
		--path . \
		--root . \
		--no-track \
		--profile release \
		--target wasm32v1-none

clean:
	rm -rf bin
