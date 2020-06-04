all: monitor

monitor:
	wasm-pack build --target no-modules
	rsync -az ./pkg/kusa_monitor*.{js,wasm} ./frontend/pkg/

dist: monitor
	rsync -az ./frontend/index.html ./dist/
	rsync -az ./frontend/plotters ./dist/
	rsync -az ./frontend/deps ./dist/
	rsync -az ./pkg/kusa_monitor*.{js,wasm} ./dist/pkg/

clean:
	rm -rf ./frontend/pkg/* ./dist/*
