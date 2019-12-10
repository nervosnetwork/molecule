MOLC = ../../tools/compiler/target/release/moleculec

SCHEMA = ci_tests
SCHEMA_FILE = schemas/${SCHEMA}.mol

TMPDIR = tmp
TMP = ${TMPDIR} ${BINS} ${MOLC} \
	  ${HEADER_API} ${HEADER_GEN}

MOL_DEPS = refresh-comipler ${MOLC}

clean:
	@cargo clean
	@rm -rf ${TMP}

tmpdir:
	@if [ ! -d "${TMPDIR}" ]; then mkdir -p "${TMPDIR}"; fi

refresh-comipler:
	@rm -f ${MOLC}

debug:
	@cargo build

test: test-rust

test-rust:
	@cargo test --all

${MOLC}:
	@cd ../../tools/compiler; cargo build --release

${HEADER_API}: ${SCHEMA_FILE} ${MOL_DEPS}
	@"${MOLC}" --language c --schema-file $< > $@

${HEADER_GEN}: ${SCHEMA_FILE}
	@${SCRIPT_GEN_C} "${SCHEMA_FILE}" "${HEADER_GEN}"
