
NPROC?=$(shell nproc)
CC=clang-16
LLVM_PROFDATA=llvm-profdata-16
LLVM_COV=llvm-cov-16
CORPUS_DIR=corpus

FUZZER_FLAGS=-g -O1 -fsanitize=fuzzer,address,undefined -I ../../bindings/c/include -I .

COVERAGE_DIR=coverage_dir
COVERAGE_FLAGS=-g -fprofile-instr-generate -fcoverage-mapping -I ../../bindings/c/include -I .

EXTERNAL_HEADERS=./fuzzer_func.h ../../bindings/c/include/molecule_reader.h

all: fuzzer coverage

show: $(COVERAGE_DIR)/fuzzer.profdata
	$(LLVM_COV) show --instr-profile=$(COVERAGE_DIR)/fuzzer.profdata coverage

report: $(COVERAGE_DIR)/fuzzer.profdata coverage $(EXTERNAL_HEADERS)
	$(LLVM_COV) report --show-functions --instr-profile=$(COVERAGE_DIR)/fuzzer.profdata coverage $(EXTERNAL_HEADERS)

fuzzer: $(EXTERNAL_HEADERS)
	$(CC) $(FUZZER_FLAGS) fuzzer.c -o fuzzer

coverage: $(EXTERNAL_HEADERS)
	$(CC) $(COVERAGE_FLAGS) coverage.c fuzzer.c -o coverage

start-fuzzer: fuzzer
	./fuzzer -max_len=255 -workers=$(NPROC) -jobs=$(NPROC) corpus

clean:
	rm -rf fuzzer coverage

gen:
	moleculec --language c --schema-file definitions.mol | clang-format > definitions.h

manually-gen:
	$(CC) -I ../bindings/c/include -I . manually-gen.c -o manually-gen
	./manually-gen

fmt:
	clang-format -style="{BasedOnStyle: google, IndentWidth: 4, SortIncludes: false}" -i *.c *.h

%.profraw: coverage
	LLVM_PROFILE_FILE=$@ ./coverage $(CORPUS_DIR)/*

%.profdata: %.profraw
	$(LLVM_PROFDATA) merge --sparse $< -o $@

.PHONY: all fuzzer coverage report

.PRECIOUS: $(COVERAGE_DIR)/fuzzer.profraw $(COVERAGE_DIR)/fuzzer.profdata
