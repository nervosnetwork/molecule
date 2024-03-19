
#ifndef _FUZZER_FUZZER_FUNC_H_
#define _FUZZER_FUZZER_FUNC_H_
#include <stdint.h>
#include <stdio.h>
#include <assert.h>

#include "definitions.h"

void access_seg(mol_seg_t seg, int* result) {
    if (seg.size > 0) {
        *result += (int)seg.ptr[0];
        *result += (int)seg.ptr[seg.size - 1];
    }
}

void access_array(mol_seg_t seg, int* result) {
    if (MolReader_ArrayType_verify(&seg, false) == 0) {
        mol_seg_t first = MolReader_ArrayType_get_nth0(&seg);
        mol_seg_t last = MolReader_ArrayType_get_nth2(&seg);
        access_seg(first, result);
        access_seg(last, result);
    }
}

void access_struct(mol_seg_t seg, int* result) {
    if (MolReader_StructType_verify(&seg, false) == 0) {
        mol_seg_t f1 = MolReader_StructType_get_f1(&seg);
        mol_seg_t f2 = MolReader_StructType_get_f2(&seg);
        access_seg(f1, result);
        access_seg(f2, result);
    }
}

void access_fixvec(mol_seg_t seg, int* result) {
    if (MolReader_FixVecType_verify(&seg, false) == 0) {
        mol_seg_t data = MolReader_FixVecType_raw_bytes(&seg);
        access_seg(data, result);
    }
}

void access_dynvec(mol_seg_t seg, int* result) {
    if (MolReader_DynVecType_verify(&seg, false) == 0) {
        uint32_t length = MolReader_DynVecType_length(&seg);
        for (uint32_t i = 0; i < length; i++) {
            mol_seg_res_t element = MolReader_DynVecType_get(&seg, i);
            access_seg(element.seg, result);
        }
        mol_seg_res_t out_of_bound = MolReader_DynVecType_get(&seg, length);
        assert(out_of_bound.errno != 0);
    }
}

void access_opt(mol_seg_t seg, int* result) {
    if (MolReader_OptType_verify(&seg, false) == 0) {
        if (MolReader_OptType_is_none(&seg)) {
            return;
        } else {
            access_dynvec(seg, result);
        }
    }
}

void access_table(mol_seg_t seg, int* result) {
    if (MolReader_TableType_verify(&seg, false) == 0) {
        mol_seg_t fixvec = MolReader_TableType_get_f1(&seg);
        access_fixvec(fixvec, result);
        mol_seg_t dynvec = MolReader_TableType_get_f2(&seg);
        access_dynvec(dynvec, result);
        mol_seg_t struct_ = MolReader_TableType_get_f3(&seg);
        access_struct(struct_, result);
        mol_seg_t array = MolReader_TableType_get_f4(&seg);
        access_array(array, result);
        mol_seg_t opt = MolReader_TableType_get_f5(&seg);
        access_opt(opt, result);
    }
}

void access_union(mol_seg_t seg, int* result) {
    if (MolReader_UnionType_verify(&seg, false) == 0) {
        mol_union_t data = MolReader_UnionType_unpack(&seg);
        access_seg(data.seg, result);
    }
}

int start_fuzzing(uint8_t* data, size_t size) {
    int result = 0;
    mol_seg_t seg = {.ptr = data, .size = size};

    access_array(seg, &result);
    access_struct(seg, &result);
    access_fixvec(seg, &result);
    access_dynvec(seg, &result);
    access_table(seg, &result);
    access_union(seg, &result);

    return result;
}

#endif
