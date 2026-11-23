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

static inline int mol_integer_compare_limbs(const uint64_t *left,
        const uint64_t *right, unsigned int count) {
    for (unsigned int i = count; i > 0; i--) {
        if (left[i - 1] < right[i - 1]) return -1;
        if (left[i - 1] > right[i - 1]) return 1;
    }
    return 0;
}

static inline int mol_uint128_compare(mol_uint128_t left, mol_uint128_t right) {
    return mol_integer_compare_limbs(left.limbs, right.limbs, 2);
}

static inline int mol_int128_compare(mol_int128_t left, mol_int128_t right) {
    /* Flip the sign bit of the most-significant limb to compare two's-complement
     * signed 128-bit values using only unsigned arithmetic. */
    uint64_t sign = (uint64_t)1 << 63;
    uint64_t l[2] = {left.limbs[0], left.limbs[1] ^ sign};
    uint64_t r[2] = {right.limbs[0], right.limbs[1] ^ sign};
    return mol_integer_compare_limbs(l, r, 2);
}

static inline mol_uint256_t mol_uint256_from_u64(uint64_t value) {
    mol_uint256_t result = {{value, 0, 0, 0}};
    return result;
}

static inline mol_uint256_t mol_uint256_from_u128(mol_uint128_t value) {
    mol_uint256_t result = {{value.limbs[0], value.limbs[1], 0, 0}};
    return result;
}

static inline int mol_uint256_compare(mol_uint256_t left, mol_uint256_t right) {
    return mol_integer_compare_limbs(left.limbs, right.limbs, 4);
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
