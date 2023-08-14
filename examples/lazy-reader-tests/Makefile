

all: gen

gen:
	PATH=../../tools/compiler/target/debug:$PATH  moleculec --schema-file ../../test/schemas/types.mol --language rust-lazy-reader > src/types_api2.rs
	PATH=../../tools/compiler/target/debug:$PATH  moleculec --schema-file ../../test/schemas/types.mol --language rust > src/types_api.rs
	cargo fmt

test:
	cargo test

.PHONY: gen all


