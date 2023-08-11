

all: gen

gen:
	PATH=../../tools/compiler/target/debug:$PATH  moleculec --schema-file ../../test/schemas/types.mol --language rust-lazy-reader > src/types.rs
	cargo fmt

.PHONY: gen all


