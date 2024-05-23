
#include "fuzzer_func.h"

int LLVMFuzzerTestOneInput(uint8_t *data, size_t size) {
    return start_fuzzing(data, size);
}
