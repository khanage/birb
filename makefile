.PHONY: wasm
.SILENT: wasm

wasm:
	cargo build --release --target wasm32-unknown-unknown

	wasm-bindgen --no-typescript --target web \
	    --out-dir ./web/ \
	    --out-name "birb" \
	    ./target/wasm32-unknown-unknown/release/birb.wasm

	cp -r assets ./web/
	WASM_SERVER_RUNNER_DIRECTORY=web WASM_SERVER_RUNNER_CUSTOM_INDEX_HTML=index.html wasm-server-runner web/birb_bg.wasm
