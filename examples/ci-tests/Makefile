MOLC_DIR = ../../tools/compiler/target/release
MOLC = ${MOLC_DIR}/moleculec
MOLINC = ../../bindings/c/include
SCRIPT_GEN_C = scripts/generate-c
BIN_GEN_C_TESTS = ../tests-utils-c/target/release/molecule-tests-utils-c

SCHEMA = types
SCHEMA_FILE = ../../test/schemas/${SCHEMA}.mol

HEADER_API = c/tests-api.h
HEADER_GEN = c/tests-gen.h

TARGET_TMP_DIR = target/tmp
BINS_C_TESTS = ${TARGET_TMP_DIR}/test-vector-default ${TARGET_TMP_DIR}/test-vector-simple
TMP = ${TARGET_TMP_DIR} ${MOLC} \
	  ${HEADER_API} ${HEADER_GEN}

MOL_DEPS = refresh-comipler ${MOLC}
C_DEPS = ${MOL_DEPS} \
		 ${HEADER_API} ${HEADER_GEN} \
		 ${MOLINC}/molecule_reader.h ${MOLINC}/molecule_builder.h

CC = gcc
CXX = g++
CFLAGS = -Wall -Werror -Wno-array-bounds

clean:
	@cargo clean
	@rm -rf ${TMP}

tmpdir:
	@if [ ! -d "${TARGET_TMP_DIR}" ]; then mkdir -p "${TARGET_TMP_DIR}"; fi

refresh-comipler:
	@rm -f ${MOLC}

debug:
	@cargo build

test: test-rust test-rust-no-std test-c test-cpp test-mixed test-import

test-rust:
	@cargo test --all

test-rust-no-std:
	@cargo test --all --no-default-features

test-c: tmpdir ${TARGET_TMP_DIR}/test-build-default ${BIN_GEN_C_TESTS} ${BINS_C_TESTS}
	@${TARGET_TMP_DIR}/test-build-default
	@for bin in ${BINS_C_TESTS} ; do "$${bin}"; done

test-cpp: tmpdir ${TARGET_TMP_DIR}/test-build-default-cpp
	@${TARGET_TMP_DIR}/test-build-default-cpp

test-mixed: tmpdir ${TARGET_TMP_DIR}/simple-example
	@tmpdir="${TARGET_TMP_DIR}" scripts/test-mixed simple-example

test-import: test-rust-import test-c-import

test-rust-import: tmpdir ${MOL_DEPS}
	@export PATH="$$(cd ${MOLC_DIR}; pwd):$${PATH}" \
		; tmpdir="${TARGET_TMP_DIR}" molc="${MOLC}" schemas_dir="schemas/import" scripts/test-import-gen-code rust
	@cd "${TARGET_TMP_DIR}/import/rust"; cargo build
	@echo "Passed: Test Rust Import."

test-c-import: tmpdir ${MOL_DEPS}
	@export PATH="$$(cd ${MOLC_DIR}; pwd):$${PATH}" \
		; tmpdir="${TARGET_TMP_DIR}" molc="${MOLC}" schemas_dir="schemas/import" scripts/test-import-gen-code c
	@${CC} ${CFLAGS} -I${MOLINC} -c "${TARGET_TMP_DIR}/import/c/b/test.c" -o "${TARGET_TMP_DIR}/import/c/b/test.o"
	@echo "Passed: Test C Import."

${MOLC}:
	@cd ../../tools/compiler; cargo build --release

${BIN_GEN_C_TESTS}:
	@cd ../tests-utils-c; cargo build --release

${HEADER_API}: ${SCHEMA_FILE} ${MOL_DEPS}
	@set -eu \
		; export PATH="$$(cd ${MOLC_DIR}; pwd):$${PATH}" \
		; "${MOLC}" --language c --schema-file $< > $@

${HEADER_GEN}: ${SCHEMA_FILE}
	@${SCRIPT_GEN_C} "${SCHEMA_FILE}" "${HEADER_GEN}"

${TARGET_TMP_DIR}/test-build-default: c/test-build-default.c ${C_DEPS}
	@${CC} ${CFLAGS} -I${MOLINC} -o $@ $<

${TARGET_TMP_DIR}/test-build-default-cpp: c/test-build-default.c ${C_DEPS}
	@${CXX} ${CFLAGS} -I${MOLINC} -o $@ $<

${TARGET_TMP_DIR}/simple-example: c/simple-example.c ${C_DEPS}
	@${CC} ${CFLAGS} -I${MOLINC} -o $@ $<

c/test-vector-default.c: ${BIN_GEN_C_TESTS}
	@${BIN_GEN_C_TESTS} ../../test/schemas/types.mol ../../test/vectors/default.yaml > $@

c/test-vector-simple.c: ${BIN_GEN_C_TESTS}
	@${BIN_GEN_C_TESTS} ../../test/schemas/types.mol ../../test/vectors/simple.yaml > $@

${TARGET_TMP_DIR}/test-vector-default: c/test-vector-default.c ${C_DEPS}
	@${CC} ${CFLAGS} -I${MOLINC} -o $@ $<

${TARGET_TMP_DIR}/test-vector-simple: c/test-vector-simple.c ${C_DEPS}
	@${CC} ${CFLAGS} -I${MOLINC} -o $@ $<
