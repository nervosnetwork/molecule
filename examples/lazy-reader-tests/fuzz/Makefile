

# To accelerate the fuzzing process, please download the pre-made corpus file
# from the following link:
# https://gist.github.com/joii2020/ce55af204f946eebd83269ef8da3f3d1 Save the
# file named corpus.bin to the corpus/fuzz_target_1/ folder.

all:
	cargo fuzz run -j30 fuzz_target_1

DEFAUTL_TARGET = `rustup show | grep 'Default host' | grep -o ': .*' | sed 's/: //'`
cov:
	cargo fuzz coverage fuzz_target_1
	cargo-cov -- \
		show target/$(DEFAUTL_TARGET)/coverage/$(DEFAUTL_TARGET)/release/fuzz_target_1 \
		--format=html \
		--ignore-filename-regex="(\/.*\/registry\/*)|(\/rustc\/*)|(\/types_api.rs)" \
		-instr-profile=coverage/fuzz_target_1/coverage.profdata \
			> coverage.html

install:
	rustup component add llvm-tools-preview
	cargo install cargo-fuzz
	cargo install cargo-binutils

.PHONY:
	all cov install

