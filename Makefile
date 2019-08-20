ci: ci-example ci-rust ci-c

ci-rust:
	set -eu; \
	export RUSTFLAGS='-F warnings'; \
	for dir in tools/codegen tools/compiler bindings/rust; do \
		cd "$${dir}"; \
		cargo clean; \
		cargo fmt --all -- --check; \
		cargo clippy --all --all-targets --all-features; \
		cargo test --all --verbose; \
		cd ../..; \
	done

ci-c:
	set -eu; \
	echo "TODO: not finished yet."

ci-example:
	set -eu; \
	cd examples/ci-tests; \
	make test
