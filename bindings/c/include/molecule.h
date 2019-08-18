#ifndef MOLECULE_H
#define MOLECULE_H

#ifdef __cplusplus
#define _CPP_BEGIN extern "C" {
#define _CPP_END }
_CPP_BEGIN
#endif /* __cplusplus */

#include <stddef.h>
#include <stdbool.h>
#include <stdint.h>
#include <string.h>
#include <stdarg.h>
#include <stdio.h>

/*
 * This part is not for normal users.
 */

#ifndef MOLECULE_NAMESPACE
#define _MOLECULE_NAMESPACE_IS_DEFAULT
#define MOLECULE_NAMESPACE      mol
#endif

#ifdef MOLNS
#error Please undef MOLNS before include this header file.
#endif

#define _MOLNS_CONCAT(ns,id)    ns ## _ ## id
#define _MOLNS_REPLACE(ns,id)   _MOLNS_CONCAT(ns,id)
#define MOLNS(id)               _MOLNS_REPLACE(MOLECULE_NAMESPACE, id)

// '0' ~ '9': 0x30 ~ 0x39 / 0b0011_0000 ~ 0b0011_1001
// 'A' ~ 'F': 0x41 ~ 0x46 / 0b0100_0001 ~ 0b0100_0110
// 'a' ~ 'f': 0x61 ~ 0x66 / 0b0110_0001 ~ 0b0110_0110
#define nibble(ch)              (((ch) & 0xf) + ((ch) >> 6) * 9)
#define byte(hi, lo)            ((nibble(hi) << 4) | nibble(lo))
// Test if the host is big endian machine.
#define is_le()                 (*(unsigned char *)&(uint16_t){1})

#define MOLECULE_NONE           0
#define MOLECULE_SOME           1

#define MOLECULE_TRY(res)       \
    if (MOLECULE_IS_ERR((res).code)) {              \
        return (res);                               \
    }

/*
 * Definitions of types and simple utilities.
 */

/* Core types */

typedef uint32_t                MOLNS(size_t);     // Size

// Type of data.
typedef enum {
    MolError  = 0,
    MolOption = 1,
    MolUnion  = 2,
    MolArray  = 3,
    MolStruct = 4,
    MolFixVec = 5,
    MolDynVec = 6,
    MolTable  = 7,
} MOLNS(type_t);

// Position of a struct.
typedef struct {
    const uint8_t               *ptr;               // Start position
    MOLNS(size_t)               size;               // Full size
} MOLNS(pos_t);

/* Error codes */

#define MOLECULE_OK             0x00
#define MOLECULE_ERR            0xff
#define MOLECULE_IS_OK(ec)      (MOLECULE_OK == (ec))
#define MOLECULE_IS_ERR(ec)     (MOLECULE_OK != (ec))

#define MOLECULE_ERR_OPTION     0x10
#define MOLECULE_ERR_UNION      0x20
#define MOLECULE_ERR_ARRAY      0x30
#define MOLECULE_ERR_STRUCT     0x40
#define MOLECULE_ERR_FIXVEC     0x50
#define MOLECULE_ERR_DYNVEC     0x60
#define MOLECULE_ERR_TABLE      0x70

#define MOLECULE_ERR_TOTAL_SIZE             0x01
#define MOLECULE_ERR_HEADER_IS_BROKEN       0x02
#define MOLECULE_ERR_DATA_IS_SHORT          0x03
#define MOLECULE_ERR_DATA_IS_EMPTY          0x04
#define MOLECULE_ERR_FIRST_OFFSET_IS_BROKEN 0x05
#define MOLECULE_ERR_FIRST_FIELD_IS_BROKEN  0x06
#define MOLECULE_ERR_FIELD_IS_BROKEN        0x07
#define MOLECULE_ERR_INDEX_OUT_OF_BOUNDS    0x0f

/* Result types */

typedef uint8_t                 MOLNS(res_t);      // Result code

// Result for returning bool.
typedef struct {
    MOLNS(res_t)                code;               // Result code
    bool                        ok;                 // true / false
} MOLNS(bool_res_t);

// Result for returning signed uint32.
typedef struct {
    MOLNS(res_t)                code;               // Result code
    int32_t                     num;                // Number
} MOLNS(int32_res_t);

// Result for returning unsigned uint32.
typedef struct {
    MOLNS(res_t)                code;               // Result code
    uint32_t                    num;                // Number
} MOLNS(uint32_res_t);

// Result for returning unsigned uint64.
typedef struct {
    MOLNS(res_t)                code;               // Result code
    uint64_t                    num;                // Number
} MOLNS(uint64_res_t);

// Result for returning size.
typedef struct {
    MOLNS(res_t)                code;               // Result code
    MOLNS(size_t)               size;               // Size
} MOLNS(size_res_t);

// Result for returning position.
typedef struct {
    MOLNS(res_t)                code;               // Result code
    MOLNS(pos_t)                pos;                // Position
    // Result
    //  - Ok:
    //      - For Option:
    //          - 0     : None
    //          - > 0   : Some
    //      - For Union:
    //          the inner type id
    //      - For Array, Struct:
    //          always be 0
    //      - For FixVec, DynVec, Table:
    //          total items or fields count
    //  - Error:
    //      always be 0
    uint32_t                    attr;
} MOLNS(read_res_t);

/* Utilities. */

// Returns the size of a string.
MOLNS(size_t) MOLNS(strlen) (uint8_t s[]) {
    uint8_t *p = s;
    while (*p != '\0') { p++; }
    return (p-s);
}

// Returns:
//      0   : pass the check
//      > 0 : first error position
MOLNS(size_t) MOLNS(hex_check) (uint8_t s[], uint32_t* len) {
    uint8_t *p = s;
    while (*p != '\0') {
        uint8_t ch = *p;
        if (    '0' > ch || ch > 'f'
                || ('9' < ch && ch < 'A')
                || ('F' < ch && ch < 'a') ) {
            return (p-s+1);
        }
        p++;
    }
    if ((p - s) % 2 == 0) {
        *len = p-s;
        return 0;
    } else {
        return (p-s+1);
    }
}

// Convert a nibble to a hex char.
uint8_t MOLNS(nibble2hex) (uint8_t ch) {
    return ch + (ch < 10 ? '0' : ('a' - 10));
}

// Convert a binary data to a hex string.
void MOLNS(bin2hex) (uint8_t bin[], size_t len, char hex[]) {
    for (int i=0; i<len; i++) {
        hex[i*2] = MOLNS(nibble2hex)(bin[i] >> 4);
        hex[i*2+1] = MOLNS(nibble2hex)(bin[i] & 0xf);
    }
}

// Convert a hex string to a binary data.
void MOLNS(hex2bin) (uint8_t hex[], size_t len, uint8_t bin[]) {
    for (int i=0; i<len; i++) {
        bin[i] = byte(hex[i*2], hex[i*2+1]);
    }
}

// Compare two hex strings via their binary data.
MOLNS(int32_res_t) MOLNS(hex_cmp) (uint8_t hex1[], uint8_t hex2[]) {
    uint32_t len1 = 0;
    uint32_t len2 = 0;
    uint32_t len_bin;
    MOLNS(int32_res_t) res;
    res.code = MOLNS(hex_check)(hex1, &len1);
    MOLECULE_TRY(res);
    res.code = MOLNS(hex_check)(hex2, &len2);
    MOLECULE_TRY(res);
    if (len1 < len2) {
        len_bin = len1 >> 1;
    } else {
        len_bin = len2 >> 1;
    }
    uint8_t bin1[len_bin];
    uint8_t bin2[len_bin];
    MOLNS(hex2bin)(hex1, len_bin, bin1);
    MOLNS(hex2bin)(hex2, len_bin, bin2);
    res.num = memcmp(bin1, bin2, len_bin);
    if (res.num == 0) {
        if (len1 < len2) {
            res.num = - (len_bin + 1);
        } else if (len1 > len2) {
            res.num = (len_bin + 1);
        }
    }
    return res;
}

void MOLNS(dbg_bin)(const char *format, uint8_t bin[], size_t len) {
    char hex[len*2+1];
    MOLNS(bin2hex)(bin, len*2, hex);
    hex[len*2] = '\0';
    printf(format, hex);
}

MOLNS(size_t) MOLNS(size_from_be) (const uint8_t *src) {
    if (is_le()) {
        MOLNS(size_t) output = 0;
        uint8_t *dst = (uint8_t*) &output;
        dst[3] = src[0];
        dst[2] = src[1];
        dst[1] = src[2];
        dst[0] = src[3];
        return output;
    } else {
        return *(const MOLNS(size_t) *)src;
    }
}

MOLNS(size_t) MOLNS(size_from_le) (const uint8_t *src) {
    if (is_le()) {
        return *(const MOLNS(size_t) *)src;
    } else {
        MOLNS(size_t) output = 0;
        uint8_t *dst = (uint8_t*) &output;
        dst[3] = src[0];
        dst[2] = src[1];
        dst[1] = src[2];
        dst[0] = src[3];
        return output;
    }
}

size_t MOLNS(size_into_be) (uint8_t *dst, MOLNS(size_t) input) {
    const uint8_t *src = (const uint8_t*) &input;
    if (is_le()) {
        dst[3] = src[0];
        dst[2] = src[1];
        dst[1] = src[2];
        dst[0] = src[3];
    } else {
        memcpy(dst, src, 4);
    }
    return 4;
}

size_t MOLNS(size_into_le) (uint8_t *dst, MOLNS(size_t) input) {
    const uint8_t *src = (const uint8_t*) &input;
    if (is_le()) {
        memcpy(dst, src, 4);
    } else {
        dst[3] = src[0];
        dst[2] = src[1];
        dst[1] = src[2];
        dst[0] = src[3];
    }
    return 4;
}

/*
 * Core functions.
 */

// Cut a part of bytes from a whole bytes.
// Usage:
//     - cut(&parent, MolOption)
//     - cut(&parent, MolUnion )
//     - cut(&parent, MolArray , item_count, item_size, item_index*)
//     - cut(&parent, MolStruct, total_size, field_offset, field_size)
//     - cut(&parent, MolFixVec, item_size, item_index*)
//     - cut(&parent, MolDynVec, item_index*)
//     - cut(&parent, MolTable , field_index)
MOLNS(read_res_t) MOLNS(cut) (const MOLNS(pos_t) *parent, ...) {
    MOLNS(read_res_t) res;
    res.code = MOLECULE_OK;
    res.pos.ptr = NULL;
    res.pos.size = 0;
    va_list args;
    va_start(args, parent);
    MOLNS(type_t) parent_type = va_arg(args, MOLNS(type_t));
    switch (parent_type) {
        case MolOption:
            {
                if (parent->size == 0) {
                    // None
                    res.attr = MOLECULE_NONE;
                } else {
                    // Some
                    res.pos.ptr = parent->ptr;
                    res.pos.size = parent->size;
                    res.attr = MOLECULE_SOME;
                }
            }
            break;
        case MolUnion:
            {
                if (parent->size < 4) {
                    res.code = MOLECULE_ERR_UNION | MOLECULE_ERR_HEADER_IS_BROKEN;
                } else {
                    res.attr = MOLNS(size_from_le)(parent->ptr);
                    res.pos.ptr = parent->ptr + 4;
                    res.pos.size = parent->size - 4;
                }
            }
            break;
        case MolArray:
            {
                MOLNS(size_t) item_count = va_arg(args, MOLNS(size_t));
                MOLNS(size_t) item_size = va_arg(args, MOLNS(size_t));
                MOLNS(size_t) item_index = va_arg(args, MOLNS(size_t));

                if (parent->size != item_size * item_count) {
                    res.code = MOLECULE_ERR_ARRAY | MOLECULE_ERR_TOTAL_SIZE;
                } else if (item_index >= item_count) {
                    res.code = MOLECULE_ERR_ARRAY | MOLECULE_ERR_INDEX_OUT_OF_BOUNDS;
                } else {
                    res.pos.ptr = parent->ptr + item_size * item_index;
                    res.pos.size = item_size;
                }
            }
            break;
        case MolStruct:
            {
                MOLNS(size_t) total_size = va_arg(args, MOLNS(size_t));
                MOLNS(size_t) field_offset = va_arg(args, MOLNS(size_t));
                MOLNS(size_t) field_size = va_arg(args, MOLNS(size_t));

                if (parent->size != total_size) {
                    res.code = MOLECULE_ERR_STRUCT | MOLECULE_ERR_TOTAL_SIZE;
                } else if (parent->size < field_offset + field_size) {
                    res.code = MOLECULE_ERR_STRUCT | MOLECULE_ERR_DATA_IS_SHORT;
                } else {
                    res.pos.ptr = parent->ptr + field_offset;
                    res.pos.size = field_size;
                }
            }
            break;
        case MolFixVec:
            {
                MOLNS(size_t) item_size = va_arg(args, MOLNS(size_t));
                MOLNS(size_t) item_index = va_arg(args, MOLNS(size_t));

                if (parent->size < 4) {
                    res.code = MOLECULE_ERR_FIXVEC | MOLECULE_ERR_HEADER_IS_BROKEN;
                } else {
                    res.attr = MOLNS(size_from_le)(parent->ptr);
                    if (item_index >= res.attr) {
                        res.code = MOLECULE_ERR_FIXVEC | MOLECULE_ERR_INDEX_OUT_OF_BOUNDS;
                    } else if (parent->size < 4 + item_size * (item_index + 1)) {
                        res.code = MOLECULE_ERR_FIXVEC | MOLECULE_ERR_DATA_IS_SHORT;
                    } else {
                        res.pos.ptr = parent->ptr + 4 + item_size * item_index;
                        res.pos.size = item_size;
                    }
                }
            }
            break;
        case MolDynVec:
        case MolTable:
            {
                MOLNS(size_t) field_count = 0;

                MOLNS(res_t) res_type;
                if (parent_type == MolDynVec) {
                    res_type = MOLECULE_ERR_DYNVEC;
                } else {
                    res_type = MOLECULE_ERR_TABLE;
                    field_count = va_arg(args, MOLNS(size_t));
                }

                MOLNS(size_t) index = va_arg(args, MOLNS(size_t));

                if (parent->size < 4) {
                    res.code = res_type | MOLECULE_ERR_HEADER_IS_BROKEN;
                } else if (parent->size == 4) {
                    res.code = res_type | MOLECULE_ERR_DATA_IS_EMPTY;
                } else if (parent->size < 8) {
                    res.code = res_type | MOLECULE_ERR_FIRST_OFFSET_IS_BROKEN;
                } else {
                    MOLNS(size_t) length = MOLNS(size_from_le)(parent->ptr);
                    MOLNS(size_t) first = MOLNS(size_from_le)(parent->ptr + 4);
                    res.attr = (first - 4) / 4;

                    if (res_type == MOLECULE_ERR_TABLE && res.attr != field_count) {
                        res.code = res_type | MOLECULE_ERR_HEADER_IS_BROKEN;
                    } else if (index >= res.attr) {
                        res.code = res_type | MOLECULE_ERR_INDEX_OUT_OF_BOUNDS;
                    } else if (length < parent->size) {
                        res.code = res_type | MOLECULE_ERR_DATA_IS_SHORT;
                    } else if (length < first) {
                        res.code = res_type | MOLECULE_ERR_FIRST_FIELD_IS_BROKEN;
                    } else {
                        MOLNS(size_t) index_offset = 4 + 4 * index;

                        MOLNS(size_t) start = MOLNS(size_from_le)(parent->ptr + index_offset);
                        if (parent->size < start) {
                            res.code = res_type | MOLECULE_ERR_FIELD_IS_BROKEN;
                        } else {
                            res.pos.ptr = parent->ptr + start;
                            if (res.attr == index +1) {
                                res.pos.size = length - start;
                            } else {
                                MOLNS(size_t) end = MOLNS(size_from_le)(parent->ptr + index_offset + 4);
                                res.pos.size = end - start;
                            }
                        }
                    }
                }
            }
            break;
        default:
            res.code = parent_type;
            break;
    }
    va_end(args);
    return res;
}

MOLNS(read_res_t) MOLNS(cut_bytes) (const MOLNS(pos_t) *parent) {
    MOLNS(read_res_t) res;
    if (parent->size < 4) {
        res.code = MOLECULE_ERR_HEADER_IS_BROKEN;
    } else {
        res.pos.size = MOLNS(size_from_le)(parent->ptr);
        if (parent->size < res.pos.size + 4) {
            res.code = MOLECULE_ERR_DATA_IS_SHORT;
        } else {
            res.code = MOLECULE_OK;
            res.pos.ptr = parent->ptr + 4;
        }
    }
    return res;
}

/*
 * Undef macros which are internal use only.
 */

#undef MOLECULE_TRY

#undef is_le
#undef nibble
#undef byte

#undef MOLNS
#undef _MOLNS_REPLACE
#undef _MOLNS_CONCAT

#ifdef _MOLECULE_NAMESPACE_IS_DEFAULT
#undef MOLECULE_NAMESPACE
#undef _MOLECULE_NAMESPACE_IS_DEFAULT
#endif

#ifdef __cplusplus
_CPP_END
#undef _CPP_BEGIN
#undef _CPP_END
#endif /* __cplusplus */

#endif /* MOLECULE_H */
