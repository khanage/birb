.PHONY: wasm
.SILENT: wasm

wasm:
	cargo build --release --target wasm32-unknown-unknown

	wasm-bindgen --no-typescript --target web \
	    --out-dir ./web/ \
	    --out-name "birb" \
	    ./target/wasm32-unknown-unknown/release/birb.wasm

	cp index.html ./web/
	cp -r assets ./web/

	http-server ./web/
