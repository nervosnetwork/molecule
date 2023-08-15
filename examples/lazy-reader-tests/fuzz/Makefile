

# To accelerate the fuzzing process, please download the pre-made corpus file
# from the following link:
# https://gist.github.com/XuJiandong/852905df792d34a3400b98145b0f69e7 Save the
# file named corpus.bin to the corpus/fuzz_target_1/ folder.

all:
	cargo fuzz run -j40 fuzz_target_1

cov:
	cargo fuzz coverage fuzz_target_1 

show:
	~/.rustup/toolchains/nightly-2023-07-26-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin/llvm-cov \
	show target/x86_64-unknown-linux-gnu/release/fuzz_target_1 \
    --format=html \
    --instr-profile=coverage/fuzz_target_1/coverage.profdata \
    > coverage.html

install:
	rustup component add llvm-tools-preview

.PHONY:
	all cov install

