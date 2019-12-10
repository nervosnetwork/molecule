ci: ci-example ci-rust ci-c

RUST_PROJS = examples/ci-tests examples/nostd-tests bindings/rust tools/codegen tools/compiler
C_PROJS = examples/ci-tests

clean:
	@set -eu; \
	for dir in ${RUST_PROJS}; do \
		cd "$${dir}"; \
		cargo clean; \
		cd - > /dev/null; \
	done; \
	for dir in ${C_PROJS}; do \
		cd "$${dir}"; \
		make clean; \
		cd - > /dev/null; \
	done

ci-rust:
	@set -eu; \
	export RUSTFLAGS='-F warnings'; \
	for dir in ${RUST_PROJS}; do \
		cd "$${dir}"; \
		cargo clean; \
		cargo fmt --all -- --check; \
		cargo clippy --all --all-targets --all-features; \
		cargo test --all --verbose; \
		cd - > /dev/null; \
	done

ci-c:
	@set -eu; \
	echo "TODO: not finished yet."

ci-example:
	@set -eu; \
	cd examples/ci-tests; \
	make clean test
