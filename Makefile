ci:
	@set -eu; \
	export RUSTFLAGS='-D warnings'; \
	make fmt clippy; \
	make ci-lazy-reader; \
	make cargo-test ci-examples ci-crates; \
	echo "Success!"

RUST_DEV_PROJS = examples/ci-tests examples/integer-tests tests
RUST_PROD_PROJS = bindings/rust tools/codegen tools/compiler
RUST_PROJS = ${RUST_DEV_PROJS} ${RUST_PROD_PROJS}
C_PROJS = examples/ci-tests

.PHONY: ci-prepare
# Library lockfiles are untracked. Seed them from the compiler's tested versions
# so a fresh checkout does not resolve dependencies beyond the supported MSRV.
ci-prepare:
	@set -eu; \
	for dir in bindings/rust tools/codegen examples/tests-utils-c; do \
		if [ ! -f "$${dir}/Cargo.lock" ] || [ tools/compiler/Cargo.lock -nt "$${dir}/Cargo.lock" ]; then \
			cp tools/compiler/Cargo.lock "$${dir}/Cargo.lock"; \
		fi; \
	done

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

fmt:
	@set -eu; \
	for dir in ${RUST_PROJS}; do \
		cd "$${dir}"; \
		cargo fmt --all -- --check; \
		cd - > /dev/null; \
	done

clippy: ci-prepare
	@set -eu; \
	for dir in ${RUST_PROJS}; do \
		cd "$${dir}"; \
		cargo clippy --all --all-targets --all-features; \
		cd - > /dev/null; \
	done

cargo-test: ci-prepare
	@set -eu; \
	for dir in ${RUST_PROJS}; do \
		cd "$${dir}"; \
		cargo test; \
		cd - > /dev/null; \
	done


ci-msrv: ci-prepare
	@set -eu; \
	for dir in ${RUST_PROD_PROJS}; do \
		cd "$${dir}"; \
		cargo clean; \
		cargo build --all --verbose; \
		cd - > /dev/null; \
	done; \
	git diff --exit-code tools/compiler/Cargo.lock

ci-crates: ci-prepare
	@set -eu; \
	for dir in ${RUST_PROJS}; do \
		cd "$${dir}"; \
		cargo clean; \
		cargo test --all --verbose; \
		cd - > /dev/null; \
	done; \
	git diff --exit-code tools/compiler/Cargo.lock

ci-examples: ci-prepare
	@set -eu; \
	cd examples/ci-tests; \
	make clean test; \
	cd - > /dev/null; \

ci-lazy-reader:
	@set -eu; \
	cd examples/lazy-reader-tests; \
	make test; \
	cd - > /dev/null; \


