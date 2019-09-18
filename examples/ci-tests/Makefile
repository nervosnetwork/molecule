MOLC = ../../tools/compiler/target/release/moleculec
MOLINC = ../../bindings/c/include
SCRIPT_GEN_C = scripts/generate_c

SCHEMA = ci_tests
SCHEMA_FILE = schemas/${SCHEMA}.mol

HEADER_API = c/${SCHEMA}_api.h
HEADER_GEN = c/${SCHEMA}_gen.h

BINS = ci_test_simple ci_test_build
TMP = ${BINS} ${MOLC} *.tmp \
	  ${HEADER_API} ${HEADER_GEN}

MOL_DEPS = refresh-comipler ${MOLC}
C_DEPS = ${MOL_DEPS} \
		 ${HEADER_API} ${HEADER_GEN} \
		 ${MOLINC}/molecule_reader.h ${MOLINC}/molecule_builder.h

CC = gcc
CFLAGS = -Wall -Werror

clean:
	@cargo clean
	@rm -f ${TMP}

refresh-comipler:
	@rm -f ${MOLC}

debug:
	@cargo build

test: test-rust test-c test-mixed

test-rust:
	@cargo test --all

test-c: ci_test_build
	@./ci_test_build

test-mixed: ci_test_simple
	@set -eu; \
	cargo run        | grep "^AllInOneTestData :  " > rust.tmp; \
	./ci_test_simple | grep "^AllInOneTestData :  " >    c.tmp; \
	diff rust.tmp c.tmp; \
	echo "Passed: Test Mixed."

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
