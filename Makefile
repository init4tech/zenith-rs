.PHONY: build
build:
	@cargo b --release

.PHONY: clean
clean:
	@cargo clean

.PHONY: check
check:
	@cargo check

.PHONY: test
test:
	@cargo test

.PHONY: fmt
fmt:
	@cargo +nightly fmt --all

.PHONY: clippy
clippy:
	@cargo clippy --all-targets --all-features -D warnings

tidy:
	@cargo clippy --all-targets --all-features -D warnings && @crago +nightly fmt --all