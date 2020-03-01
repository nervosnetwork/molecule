ci: ci-example ci-crates

RUST_PROJS = examples/ci-tests bindings/rust tools/codegen tools/compiler
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

ci-crates:
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

ci-example:
	@set -eu; \
	export RUSTFLAGS='-F warnings'; \
	cd examples/ci-tests; \
	make clean test
