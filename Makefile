prog = simple-roguelike

linux-build:
	cargo build --release
	strip target/release/${prog}

wasm-build:
	cargo build --release --target wasm32-unknown-unknown
	wasm-bindgen target/wasm32-unknown-unknown/release/$(prog).wasm --out-dir wasm --no-modules --no-typescript

windows-build:
	cargo build --release --target x86_64-pc-windows-gnu
	strip target/x86_64-pc-windows-gnu/release/$(prog).exe

#install:
#	cp target/$(target)/$(prog) ~/bin/$(prog)-$(extension)

build-all: linux-build wasm-build windows-build

help:
	@echo "usage: make $(prog) [debug=1]"
