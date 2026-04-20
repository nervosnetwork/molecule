RUST_PROD_PKGS = molecule molecule-codegen moleculec

ci:
	@set -eu; \
	export RUSTFLAGS='-D warnings'; \
	make fmt clippy; \
	make ci-lazy-reader; \
	make cargo-test ci-examples ci-crates; \
	echo "Success!"

clean:
	@cargo clean
	@$(MAKE) -C examples/ci-tests clean

fmt:
	@cargo fmt --all -- --check

clippy:
	@cargo clippy --all-targets --all-features -- -D warnings

cargo-test:
	@cargo test


ci-msrv:
	@set -eu; \
	cargo clean; \
	cargo build --package molecule --package molecule-codegen --package moleculec --verbose; \
	git diff --exit-code Cargo.lock

ci-crates:
	@set -eu; \
	cargo clean; \
	cargo test --verbose; \
	git diff --exit-code Cargo.lock

ci-examples:
	@$(MAKE) -C examples/ci-tests clean test

ci-lazy-reader:
	@$(MAKE) -C examples/lazy-reader-tests test
