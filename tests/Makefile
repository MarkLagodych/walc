WAT2WASM=wasm-tools parse

.PHONY: all clean

all: \
bin \
bin/minimal.wasm \
bin/simple.wasm \
bin/function.wasm \
bin/function-locals.wasm \
bin/if.wasm \
bin/br.wasm \
bin/loop.wasm \
bin/memory.wasm \
bin/add-sub.wasm \
bin/bitwise.wasm \
bin/shift.wasm \
bin/rotations.wasm \
bin/count-zeroes.wasm

bin:
	mkdir -p bin

bin/%.wasm: %.wat
	$(WAT2WASM) $< -o $@

clean:
	rm -rf bin
