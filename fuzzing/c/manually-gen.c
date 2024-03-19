#include <stdio.h>
#include <stdint.h>
#include <assert.h>
#include "definitions.h"

int write(uint8_t* data, size_t length, const char* file_name) {
    FILE* file = fopen(file_name, "wb");
    if (file == NULL) {
        fprintf(stderr, "Error: Unable to open file %s\n", file_name);
        return -1;
    }

    size_t bytes_written =
        fwrite(data, sizeof(uint8_t), length, file);  // Write data to file
    if (bytes_written != length) {
        fprintf(stderr, "Error: Unable to write all data to file\n");
        fclose(file);
        return -1;
    }
    printf("%s is generated", file_name);
    fclose(file);
    return 0;
}

int main() {
    mol_builder_t fixvec_builder = {0};
    MolBuilder_FixVecType_init(&fixvec_builder);
    MolBuilder_FixVecType_push(&fixvec_builder, 1);
    mol_seg_res_t res = MolBuilder_FixVecType_build(fixvec_builder);
    assert(res.errno == 0);
    mol_seg_t fix_vec = res.seg;

    mol_builder_t dynvec_builder = {0};
    MolBuilder_DynVecType_init(&dynvec_builder);
    MolBuilder_DynVecType_push(&dynvec_builder, fix_vec.ptr, fix_vec.size);
    res = MolBuilder_DynVecType_build(dynvec_builder);
    assert(res.errno == 0);
    mol_seg_t dyn_vec = res.seg;

    uint8_t array[3] = {0};
    uint8_t byte = 0;
    mol_builder_t struct_builder = {0};
    MolBuilder_StructType_init(&struct_builder);
    MolBuilder_StructType_set_f1(&struct_builder, array);
    MolBuilder_StructType_set_f2(&struct_builder, byte);
    res = MolBuilder_StructType_build(struct_builder);
    assert(res.errno == 0);
    mol_seg_t struct_ = res.seg;

    mol_builder_t table_builder = {0};
    MolBuilder_TableType_init(&table_builder);
    MolBuilder_TableType_set_f1(&table_builder, fix_vec.ptr, fix_vec.size);
    MolBuilder_TableType_set_f2(&table_builder, dyn_vec.ptr, dyn_vec.size);
    MolBuilder_TableType_set_f3(&table_builder, struct_.ptr, struct_.size);
    MolBuilder_TableType_set_f4(&table_builder, array, 3);
    res = MolBuilder_TableType_build(table_builder);
    assert(res.errno == 0);
    mol_seg_t table = res.seg;

    return write(table.ptr, table.size, "corpus/sample_table");
}
