WAT2WASM=wasm-tools parse

.PHONY: all clean

all: \
bin \
bin/minimal.wasm \
bin/simple.wasm \
bin/hello.wasm \
bin/function.wasm \
bin/function-locals.wasm

bin:
	mkdir -p bin

bin/%.wasm: %.wat
	$(WAT2WASM) $< -o $@

clean:
	rm -rf bin
