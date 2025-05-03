.PHONY: wasm win
.SILENT: wasm win

wasm:
	rm -rf ./web/
	mkdir web

	time cargo build --release --target wasm32-unknown-unknown

	time wasm-bindgen --no-typescript --target web \
	    --out-dir ./web/ \
	    --out-name "birb" \
	    ./target/wasm32-unknown-unknown/release/birb.wasm

	cp -r assets ./web/
	cp index.html ./web/

	http-server ./web/

win: 
	cargo build --target x86_64-pc-windows-gnu
	cp -r assets target/x86_64-pc-windows-gnu/debug/
	cargo run --target x86_64-pc-windows-gnu
