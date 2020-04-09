#include "types.h"

uint32_t tablea_verify(uint8_t *data, uint32_t data_len) {
    mol_seg_t input;
    input.ptr = data;
    input.size = data_len;
    mol_errno result = MolReader_TableA_verify(&input, false);
    return result == MOL_OK ? 0 : 1;
}
