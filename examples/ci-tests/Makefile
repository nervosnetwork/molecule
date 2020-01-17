MOLC = ../../tools/compiler/target/release/moleculec
MOLINC = ../../bindings/c/include
SCRIPT_GEN_C = scripts/generate-c

SCHEMA = types
SCHEMA_FILE = ../../test/schemas/${SCHEMA}.mol

HEADER_API = c/tests-api.h
HEADER_GEN = c/tests-gen.h

TMPDIR = target/tmp
BINS = ${TMPDIR}/test-build-default ${TMPDIR}/simple-example
TMP = ${TMPDIR} ${MOLC} \
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

test: test-rust test-rust-no-std test-c test-mixed

test-rust:
	@cargo test --all

test-rust-no-std:
	@cargo test --all --no-default-features

test-c: tmpdir ${TMPDIR}/test-build-default
	@${TMPDIR}/test-build-default

test-mixed: tmpdir ${TMPDIR}/simple-example
	@tmpdir="${TMPDIR}" scripts/test-mixed simple-example

test-import: test-rust-import test-c-import

test-rust-import: tmpdir ${MOL_DEPS}
	@tmpdir="${TMPDIR}" molc="${MOLC}" schemas_dir="schemas/import" scripts/test-import-gen-code rust
	@cd "${TMPDIR}/import/rust"; cargo build
	@echo "Passed: Test Rust Import."

test-c-import: tmpdir ${MOL_DEPS}
	@tmpdir="${TMPDIR}" molc="${MOLC}" schemas_dir="schemas/import" scripts/test-import-gen-code c
	@${CC} ${CFLAGS} -I${MOLINC} -c "${TMPDIR}/import/c/b/test.c" -o "${TMPDIR}/import/c/b/test.o"
	@echo "Passed: Test C Import."

${MOLC}:
	@cd ../../tools/compiler; cargo build --release

${HEADER_API}: ${SCHEMA_FILE} ${MOL_DEPS}
	@"${MOLC}" --language c --schema-file $< > $@

${HEADER_GEN}: ${SCHEMA_FILE}
	@${SCRIPT_GEN_C} "${SCHEMA_FILE}" "${HEADER_GEN}"

${TMPDIR}/test-build-default: c/test-build-default.c ${C_DEPS}
	@${CC} ${CFLAGS} -I${MOLINC} -o $@ $<

${TMPDIR}/simple-example: c/simple-example.c ${C_DEPS}
	@${CC} ${CFLAGS} -I${MOLINC} -o $@ $<
