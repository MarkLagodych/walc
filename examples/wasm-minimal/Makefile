WAT2WASM=wasm-tools parse

.PHONY: all clean

all: \
bin/minimal.wasm \
bin/simple.wasm

bin/%.wasm: %.wat
	$(WAT2WASM) $< -o $@

clean:
	rm -rf bin
