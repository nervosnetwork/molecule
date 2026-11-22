#ifndef MOLECULE_INTEGER_RUNTIME_INCLUDED
#define MOLECULE_INTEGER_RUNTIME_INCLUDED

#include "molecule_reader.h"

#ifdef __cplusplus
extern "C" {
#endif

/* Least-significant limb first. These native structs are never copied as wire bytes.
 * Signed 128-bit values use the same two's-complement bit representation. */
typedef struct { uint64_t limbs[2]; } mol_uint128_t;
typedef struct { uint64_t limbs[2]; } mol_int128_t;
typedef struct { uint64_t limbs[4]; } mol_uint256_t;

static inline mol_errno mol_integer_read_u64(const mol_seg_t *input,
        mol_num_t width, bool little, uint64_t *output) {
    if (!input || !output || !input->ptr || width == 0 || width > 8) return MOL_ERR_DATA;
    if (input->size != width) return MOL_ERR_TOTAL_SIZE;
    uint64_t value = 0;
    for (mol_num_t i = 0; i < width; i++) {
        mol_num_t index = little ? i : width - 1 - i;
        value |= (uint64_t)input->ptr[index] << (i * 8);
    }
    *output = value;
    return MOL_OK;
}

static inline mol_errno mol_integer_write_u64(uint64_t value, mol_num_t width,
        bool little, mol_seg_t *output) {
    if (!output || !output->ptr || width == 0 || width > 8) return MOL_ERR_DATA;
    if (output->size != width) return MOL_ERR_TOTAL_SIZE;
    if (width < 8 && value >> (width * 8)) return MOL_ERR_DATA;
    for (mol_num_t i = 0; i < width; i++) {
        mol_num_t index = little ? i : width - 1 - i;
        output->ptr[index] = (uint8_t)(value >> (i * 8));
    }
    return MOL_OK;
}

/* Stage the source limbs before writing. The native and wire buffers may overlap. */
static inline mol_errno mol_integer_read_limbs(const mol_seg_t *input,
        mol_num_t count, bool little, uint64_t *output) {
    if (!input || !input->ptr || !output || count == 0 || count > 4) return MOL_ERR_DATA;
    mol_num_t width = count * 8;
    if (input->size != width) return MOL_ERR_TOTAL_SIZE;
    uint64_t value[4];
    for (mol_num_t i = 0; i < count; i++) {
        mol_num_t offset = 8 * (little ? i : count - 1 - i);
        mol_seg_t word = {input->ptr + offset, 8};
        mol_errno error = mol_integer_read_u64(&word, 8, little, &value[i]);
        if (error != MOL_OK) return error;
    }
    for (mol_num_t i = 0; i < count; i++) output[i] = value[i];
    return MOL_OK;
}

static inline mol_errno mol_integer_write_limbs(const uint64_t *input,
        mol_num_t count, bool little, mol_seg_t *output) {
    if (!input || !output || !output->ptr || count == 0 || count > 4) return MOL_ERR_DATA;
    mol_num_t width = count * 8;
    if (output->size != width) return MOL_ERR_TOTAL_SIZE;
    uint64_t value[4];
    for (mol_num_t i = 0; i < count; i++) value[i] = input[i];
    for (mol_num_t i = 0; i < count; i++) {
        mol_num_t offset = 8 * (little ? i : count - 1 - i);
        mol_seg_t word = {output->ptr + offset, 8};
        mol_errno error = mol_integer_write_u64(value[i], 8, little, &word);
        if (error != MOL_OK) return error;
    }
    return MOL_OK;
}

static inline int mol_uint256_compare(mol_uint256_t left, mol_uint256_t right) {
    for (unsigned int i = 4; i > 0; i--) {
        if (left.limbs[i - 1] < right.limbs[i - 1]) return -1;
        if (left.limbs[i - 1] > right.limbs[i - 1]) return 1;
    }
    return 0;
}

static inline mol_errno mol_uint256_to_u128(mol_uint256_t value, mol_uint128_t *output) {
    if (!output || value.limbs[2] || value.limbs[3]) return MOL_ERR_DATA;
    output->limbs[0] = value.limbs[0];
    output->limbs[1] = value.limbs[1];
    return MOL_OK;
}

static inline mol_errno mol_uint256_to_u64(mol_uint256_t value, uint64_t *output) {
    if (!output || value.limbs[1] || value.limbs[2] || value.limbs[3]) return MOL_ERR_DATA;
    *output = value.limbs[0];
    return MOL_OK;
}

#ifdef __cplusplus
}
#endif
#endif
