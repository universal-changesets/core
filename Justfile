bacon:
	bacon

bacon-test:
	bacon test

install:
    cargo install --locked --path .

build-release:
	cargo build --release
