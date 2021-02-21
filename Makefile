all: build-release

ESP = ./node_modules/.bin/es-pack
tts:
	make release && $(ESP) build --rustwasm -m esm --dev-with-tts
build-release:
	make release && $(ESP) build --rustwasm -m esm -d ./frontend/build
build-dev:
	make dev && $(ESP) build --rustwasm -m esm --dev

dev:
	RUSTFLAGS="-Z macro-backtrace" wasm-pack build --dev --target no-modules
release:
	wasm-pack build --release --target no-modules
test:
	wasm-pack test --headless --chrome


dist: build-release
	rsync -az ./frontend/index.html ./dist/
	rsync -az ./frontend/plotters ./dist/
	rsync -az ./frontend/build ./dist/
.PHONY: dist

serve:
	# e.g. _site/frontend/index.html -> http://127.0.0.1:4000/frontend/index.html
	bundle exec jekyll serve

clean:
	rm -rf ./frontend/pkg/* ./dist/*
