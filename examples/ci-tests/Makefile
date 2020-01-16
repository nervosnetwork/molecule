MOLC = ../../tools/compiler/target/release/moleculec
MOLINC = ../../bindings/c/include
SCRIPT_GEN_C = scripts/generate_c

SCHEMA = ci_tests
SCHEMA_FILE = schemas/${SCHEMA}.mol

HEADER_API = c/${SCHEMA}_api.h
HEADER_GEN = c/${SCHEMA}_gen.h

BINS = ci_test_simple ci_test_build
TMPDIR = tmp
TMP = ${TMPDIR} ${BINS} ${MOLC} \
	  ${HEADER_API} ${HEADER_GEN}

MOL_DEPS = refresh-comipler ${MOLC}
C_DEPS = ${MOL_DEPS} \
		 ${HEADER_API} ${HEADER_GEN} \
		 ${MOLINC}/molecule_reader.h ${MOLINC}/molecule_builder.h

CC = gcc
CFLAGS = -Wall -Werror

clean:
	@cargo clean
	@rm -rf ${TMP}

tmpdir:
	@if [ ! -d "${TMPDIR}" ]; then mkdir -p "${TMPDIR}"; fi

refresh-comipler:
	@rm -f ${MOLC}

debug:
	@cargo build

test: test-rust test-rust-no-std test-c test-mixed test-import

test-rust:
	@cargo test --all

test-rust-no-std:
	@cargo test --all --no-default-features

test-c: ci_test_build
	@./ci_test_build

test-mixed: tmpdir ci_test_simple
	@set -eu; \
	cargo run                       | grep "^AllInOneTestData :  " > "${TMPDIR}/testdata-rust"; \
	cargo run --no-default-features | grep "^AllInOneTestData :  " > "${TMPDIR}/testdata-rust-no-std"; \
	./ci_test_simple                | grep "^AllInOneTestData :  " > "${TMPDIR}/testdata-c"; \
	diff "${TMPDIR}/testdata-rust" "${TMPDIR}/testdata-rust-no-std"; \
	diff "${TMPDIR}/testdata-rust" "${TMPDIR}/testdata-c"; \
	echo "Passed: Test Mixed."

test-import: test-rust-import test-c-import

test-rust-import: tmpdir ${MOL_DEPS}
	@mkdir -p \
		"${TMPDIR}/import/rust/src/a" \
		"${TMPDIR}/import/rust/src/b/bb" \
		"${TMPDIR}/import/rust/src/b/bc" \
		"${TMPDIR}/import/rust/src/c/cc"
	@"${MOLC}" --language rust --schema-file schemas/import/a/a.mol     >  "${TMPDIR}/import/rust/src/a/a.rs"
	@"${MOLC}" --language rust --schema-file schemas/import/b/b.mol     >  "${TMPDIR}/import/rust/src/b/b.rs"
	@"${MOLC}" --language rust --schema-file schemas/import/c/c.mol     >  "${TMPDIR}/import/rust/src/c/c.rs"
	@"${MOLC}" --language rust --schema-file schemas/import/b/bb/bb.mol >  "${TMPDIR}/import/rust/src/b/bb/bb.rs"
	@"${MOLC}" --language rust --schema-file schemas/import/b/bc/bc.mol >  "${TMPDIR}/import/rust/src/b/bc/bc.rs"
	@"${MOLC}" --language rust --schema-file schemas/import/c/cc/cc.mol >  "${TMPDIR}/import/rust/src/c/cc/cc.rs"
	@"${MOLC}" --language rust --schema-file schemas/import/b/test.mol  >  "${TMPDIR}/import/rust/src/b/test.rs"
	@echo "pub mod a;"                                                  >  "${TMPDIR}/import/rust/src/lib.rs"
	@echo "pub mod b;"                                                  >> "${TMPDIR}/import/rust/src/lib.rs"
	@echo "pub mod c;"                                                  >> "${TMPDIR}/import/rust/src/lib.rs"
	@echo "pub mod a;"                                                  >  "${TMPDIR}/import/rust/src/a/mod.rs"
	@echo "pub mod b;"                                                  >  "${TMPDIR}/import/rust/src/b/mod.rs"
	@echo "pub mod c;"                                                  >  "${TMPDIR}/import/rust/src/c/mod.rs"
	@echo "pub mod bb;"                                                 >  "${TMPDIR}/import/rust/src/b/bb/mod.rs"
	@echo "pub mod bb;"                                                 >> "${TMPDIR}/import/rust/src/b/mod.rs"
	@echo "pub mod bc;"                                                 >  "${TMPDIR}/import/rust/src/b/bc/mod.rs"
	@echo "pub mod bc;"                                                 >> "${TMPDIR}/import/rust/src/b/mod.rs"
	@echo "pub mod cc;"                                                 >  "${TMPDIR}/import/rust/src/c/cc/mod.rs"
	@echo "pub mod cc;"                                                 >> "${TMPDIR}/import/rust/src/c/mod.rs"
	@echo "pub mod test;"                                               >> "${TMPDIR}/import/rust/src/b/mod.rs"
	@echo '[package]'                                                   >  "${TMPDIR}/import/rust/Cargo.toml"
	@echo 'name = "molecule-test-impot"'                                >> "${TMPDIR}/import/rust/Cargo.toml"
	@echo 'version = "0.1.0"'                                           >> "${TMPDIR}/import/rust/Cargo.toml"
	@echo 'edition = "2018"'                                            >> "${TMPDIR}/import/rust/Cargo.toml"
	@echo '[dependencies]'                                              >> "${TMPDIR}/import/rust/Cargo.toml"
	@echo 'molecule = { path = "../../../../../bindings/rust" }'        >> "${TMPDIR}/import/rust/Cargo.toml"
	@cd "${TMPDIR}/import/rust"; cargo build
	@echo "Passed: Test Rust Import."

test-c-import: tmpdir ${MOL_DEPS}
	@mkdir -p \
		"${TMPDIR}/import/c/a" \
		"${TMPDIR}/import/c/b/bb" \
		"${TMPDIR}/import/c/b/bc" \
		"${TMPDIR}/import/c/c/cc"
	@"${MOLC}" --language c --schema-file schemas/import/a/a.mol     > "${TMPDIR}/import/c/a/a.h"
	@"${MOLC}" --language c --schema-file schemas/import/b/b.mol     > "${TMPDIR}/import/c/b/b.h"
	@"${MOLC}" --language c --schema-file schemas/import/c/c.mol     > "${TMPDIR}/import/c/c/c.h"
	@"${MOLC}" --language c --schema-file schemas/import/b/bb/bb.mol > "${TMPDIR}/import/c/b/bb/bb.h"
	@"${MOLC}" --language c --schema-file schemas/import/b/bc/bc.mol > "${TMPDIR}/import/c/b/bc/bc.h"
	@"${MOLC}" --language c --schema-file schemas/import/c/cc/cc.mol > "${TMPDIR}/import/c/c/cc/cc.h"
	@"${MOLC}" --language c --schema-file schemas/import/b/test.mol  > "${TMPDIR}/import/c/b/test.c"
	@${CC} ${CFLAGS} -I${MOLINC} -c "${TMPDIR}/import/c/b/test.c" -o "${TMPDIR}/import/c/b/test.o"
	@echo "Passed: Test C Import."

${MOLC}:
	@cd ../../tools/compiler; cargo build --release

${HEADER_API}: ${SCHEMA_FILE} ${MOL_DEPS}
	@"${MOLC}" --language c --schema-file $< > $@

${HEADER_GEN}: ${SCHEMA_FILE}
	@${SCRIPT_GEN_C} "${SCHEMA_FILE}" "${HEADER_GEN}"

ci_test_build: c/ci_test_build.c ${C_DEPS}
	@${CC} ${CFLAGS} -I${MOLINC} -o $@ $<

ci_test_simple: c/ci_test_simple.c ${C_DEPS}
	@${CC} ${CFLAGS} -I${MOLINC} -o $@ $<
