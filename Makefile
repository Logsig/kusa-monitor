all: monitor

monitor:
	wasm-pack build --target no-modules
	echo 'self.wasm_bindgen = wasm_bindgen;' >> ./pkg/kusa_monitor.js  # Hack for the missing `wasm_bindgen` error on Safari
	rsync -az ./pkg/kusa_monitor*.{js,wasm} ./frontend/pkg/

dist: monitor
	rsync -az ./frontend/index.html ./dist/
	rsync -az ./pkg/kusa_monitor*.{js,wasm} ./dist/pkg/

clean:
	rm -rf ./frontend/pkg/* ./dist/*
