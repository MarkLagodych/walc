.PHONY: all clean

TARGET=wasm32v1-none

all:
	cargo install \
		--quiet \
		--path . \
		--root . \
		--no-track \
		--profile release \
		--target ${TARGET}

clean:
	rm -rf bin
