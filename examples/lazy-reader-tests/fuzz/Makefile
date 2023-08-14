
all:
	cargo fuzz run -j40 fuzz_target_1

cov:
	cargo fuzz coverage fuzz_target_1 
	cargo cov -- show target/x86_64-unknown-linux-gnu/release/fuzz_target_1 \
    --format=html \
    -instr-profile=coverage/fuzz_target_1/coverage.profdata \
    > coverage.html

install:
	rustup component add llvm-tools-preview

.PHONY:
	all cov install

