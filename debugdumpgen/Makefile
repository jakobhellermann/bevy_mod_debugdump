.PHONY: wasmbindgen
wasmbindgen: out/debugdumpgen.wasm
	wasm-bindgen out/debugdumpgen.wasm --out-dir out --target web
	wasm-opt out/debugdumpgen_bg.wasm -o out/debugdumpgen_bg.wasm -O2

out/debugdumpgen.wasm: src/lib.rs Cargo.toml
	cargo build -Zunstable-options --out-dir out --release

.PHONY: clean
clean:
	rm -rf out
