#include "tests-utils.h"

#define test_build_default_for(Name)                                    \
    {                                                                   \
        total_cnt += 1;                                                 \
        uint32_t size = sizeof(MolDefault_ ## Name);                    \
        char *name = #Name;                                             \
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
    test_start("Build Default");

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

int main(int argc, char *argv[]) {
    test_build_default();
    return 0;
}
