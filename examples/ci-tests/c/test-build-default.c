#include "tests-utils.h"

#define test_build_default_for(Name)                                    \
    {                                                                   \
        total_cnt += 1;                                                 \
        uint32_t size = sizeof(MolDefault_ ## Name);                    \
        char const *name = #Name;                                       \
        const uint8_t *expected = MolDefault_ ## Name;                  \
        mol_builder_t b;                                                \
        MolBuilder_ ## Name ## _init(&b);                               \
        mol_seg_res_t res = MolBuilder_ ## Name ## _build(b);           \
        if (res.errno != MOL_OK) {                                      \
            printf("Error %s: failed to build default\n", name);        \
            failed_cnt += 1;                                            \
        } else if (res.seg.size != size) {                              \
            printf("Error %s: default size is not match (%d != %d)\n",  \
                    name, res.seg.size, size);                          \
            failed_cnt += 1;                                            \
        } else if (memcmp(res.seg.ptr, expected, size) != 0) {          \
            printf("Error %s: default content is not match\n", name);   \
            failed_cnt += 1;                                            \
        }                                                               \
        mol_errno errno = MolReader_ ## Name ## _verify(&res.seg,false);\
        if (errno != MOL_OK) {                                          \
            printf("Error %s: failed to verify default (%d)\n",         \
                    name, errno);                                       \
            failed_cnt += 1;                                            \
        }                                                               \
        free(res.seg.ptr);                                              \
    }

void test_build_default() {
    test_start((char*)"Build Default");

    uint32_t failed_cnt = 0;
    uint32_t total_cnt = 0;

    testset_apply_all(test_build_default_for);

    if (failed_cnt == 0) {
        printf("ALL checks are passed (%d).\n", total_cnt);
    } else {
        printf("[Error] %d/%d checks are failed.\n", failed_cnt, total_cnt);
        exit(1);
    }
}

void test_fixvec_size_overflow() {
    uint8_t data[8] = {0};
    mol_seg_t seg = {data, sizeof(data)};
    mol_num_t count = 2;
    mol_pack_number(data, &count);
    assert(mol_fixvec_verify(&seg, 2) == MOL_OK);

    // The item payload size wraps to four bytes in 32-bit arithmetic.
    count = UINT32_C(0x80000002);
    mol_pack_number(data, &count);
    assert(mol_fixvec_verify(&seg, 2) == MOL_ERR_TOTAL_SIZE);

    // The largest item size and count must not wrap to a small valid size.
    count = UINT32_MAX;
    mol_pack_number(data, &count);
    seg.size = 5;
    assert(mol_fixvec_verify(&seg, UINT32_MAX) == MOL_ERR_TOTAL_SIZE);

    count = 0;
    mol_pack_number(data, &count);
    seg.size = 4;
    assert(mol_fixvec_verify(&seg, 4) == MOL_OK);
}

int main(int argc, char *argv[]) {
    test_build_default();
    test_fixvec_size_overflow();
    return 0;
}
