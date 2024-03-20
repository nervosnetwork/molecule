#include "tests-utils.h"

#define test_build_simple_for(Name)                                     \
    {                                                                   \
        total_cnt += 1;                                                 \
        uint32_t size = sizeof(expected);                               \
        char *name = #Name;                                             \
        if (res.errno != MOL_OK) {                                      \
            printf("Error %s: failed to build\n", name);                \
            failed_cnt += 1;                                            \
        } else if (res.seg.size != size) {                              \
            printf("Error %s: size is not match (%d != %d)\n",          \
                    name, res.seg.size, size);                          \
            failed_cnt += 1;                                            \
        } else if (memcmp(res.seg.ptr, expected, size) != 0) {          \
            printf("Error %s: content is not match\n", name);           \
            failed_cnt += 1;                                            \
        }                                                               \
        mol_errno errno = MolReader_ ## Name ## _verify(&res.seg,false);\
        if (errno != MOL_OK) {                                          \
            printf("Error %s: failed to verify (%d)\n", name, errno);   \
            failed_cnt += 1;                                            \
        }                                                               \
    }

void test_build_simple() {
    test_start("Build Simple");

    uint32_t failed_cnt = 0;
    uint32_t total_cnt = 0;

    mol_builder_t b;
    mol_seg_res_t res;

    mol_seg_t byte3;
    {
        const uint8_t expected[] = {
            ____, 0x12, ____,
        };
        MolBuilder_Byte3_init(&b);
        MolBuilder_Byte3_set_nth1(&b, 0x12);
        res = MolBuilder_Byte3_build(b);
        test_build_simple_for(Byte3);
        byte3 = res.seg;
    }

    mol_seg_t structb;
    {
        const uint8_t expected[] = {
            ____, 0x34, ____, ____,
            ____, 0x12, ____,
        };
        MolBuilder_StructB_init(&b);
        MolBuilder_StructB_set_f2(&b, 0x34);
        MolBuilder_StructB_set_f4(&b, byte3.ptr);
        res = MolBuilder_StructB_build(b);
        test_build_simple_for(StructB);
        structb = res.seg;
    }

    mol_seg_t bytes;
    {
        const uint8_t expected[] = {
            0x03, ____, ____, ____,
            0x12, 0x34, 0x56,
        };
        MolBuilder_Bytes_init(&b);
        MolBuilder_Bytes_push(&b, 0x12);
        MolBuilder_Bytes_push(&b, 0x34);
        MolBuilder_Bytes_push(&b, 0x56);
        res = MolBuilder_Bytes_build(b);
        test_build_simple_for(Bytes);
        bytes = res.seg;
    }

    mol_seg_t byte3vec;
    {
        const uint8_t expected[] = {
            0x03, ____, ____, ____,
            ____, 0x12, ____,
            ____, 0x12, ____,
            ____, 0x12, ____,
        };
        MolBuilder_Byte3Vec_init(&b);
        MolBuilder_Byte3Vec_push(&b, byte3.ptr);
        MolBuilder_Byte3Vec_push(&b, byte3.ptr);
        MolBuilder_Byte3Vec_push(&b, byte3.ptr);
        res = MolBuilder_Byte3Vec_build(b);
        test_build_simple_for(Byte3Vec);
        byte3vec = res.seg;
    }

    mol_seg_t bytesvec;
    {
        const uint8_t expected[] = {
            0x25, ____, ____, ____,
            0x10, ____, ____, ____,
            0x17, ____, ____, ____,
            0x1e, ____, ____, ____,
            0x03, ____, ____, ____,
            0x12, 0x34, 0x56,
            0x03, ____, ____, ____,
            0x12, 0x34, 0x56,
            0x03, ____, ____, ____,
            0x12, 0x34, 0x56,
        };
        MolBuilder_BytesVec_init(&b);
        MolBuilder_BytesVec_push(&b, bytes.ptr, bytes.size);
        MolBuilder_BytesVec_push(&b, bytes.ptr, bytes.size);
        MolBuilder_BytesVec_push(&b, bytes.ptr, bytes.size);
        res = MolBuilder_BytesVec_build(b);
        test_build_simple_for(BytesVec);
        bytesvec = res.seg;
    }

    mol_seg_t bytesopt;
    {
        const uint8_t expected[] = {
            0x03, ____, ____, ____,
            0x12, 0x34, 0x56,
        };
        MolBuilder_BytesOpt_init(&b);
        MolBuilder_BytesOpt_set(&b, bytes.ptr, bytes.size);
        res = MolBuilder_BytesOpt_build(b);
        test_build_simple_for(BytesOpt);
        bytesopt = res.seg;
    }

    mol_seg_t wordsopt;
    {
        const uint8_t expected[] = {};
        MolBuilder_WordsOpt_init(&b);
        res = MolBuilder_WordsOpt_build(b);
        test_build_simple_for(WordsOpt);
        wordsopt = res.seg;
    }

    mol_seg_t uniona;
    {
        const uint8_t expected[] = {
            0x02, ____, ____, ____,
            0x12,
        };
        MolBuilder_UnionA_init(&b);
        MolBuilder_UnionA_set_byte(&b, 0x12);
        res = MolBuilder_UnionA_build(b);
        test_build_simple_for(UnionA);
    }
    {
        const uint8_t expected[] = {
            0x0b, ____, ____, ____,
            0x03, ____, ____, ____,
            0x12, 0x34, 0x56,
        };
        MolBuilder_UnionA_init(&b);
        MolBuilder_UnionA_set_Bytes(&b, bytes.ptr, bytes.size);
        res = MolBuilder_UnionA_build(b);
        test_build_simple_for(UnionA);
        uniona = res.seg;
    }

    mol_seg_t allinone;
    {
        MolBuilder_AllInOne_init(&b);
        MolBuilder_AllInOne_set_f0(&b, 0x12);
        MolBuilder_AllInOne_set_f2(&b, byte3.ptr, byte3.size);
        MolBuilder_AllInOne_set_f29(&b, structb.ptr, structb.size);
        MolBuilder_AllInOne_set_f41(&b, bytes.ptr, bytes.size);
        MolBuilder_AllInOne_set_f43(&b, byte3vec.ptr, byte3vec.size);
        MolBuilder_AllInOne_set_f48(&b, bytesvec.ptr, bytesvec.size);
        MolBuilder_AllInOne_set_f61(&b, bytesopt.ptr, bytesopt.size);
        MolBuilder_AllInOne_set_f62(&b, wordsopt.ptr, wordsopt.size);
        MolBuilder_AllInOne_set_f72(&b, uniona.ptr, uniona.size);
        res = MolBuilder_AllInOne_build(b);
        allinone = res.seg;
    }

    if (failed_cnt == 0) {
        printf("ALL checks are passed (%d).\n", total_cnt);

        printf("\n");

        for (int i=0; i<allinone.size; i++) {
            if (i % 32 == 0) {
                printf("\nAllInOneTestData :  ");
            }
            printf("%02x", allinone.ptr[i]);
        }
        printf("\n");
    } else {
        printf("[Error] %d/%d checks are failed.\n", failed_cnt, total_cnt);
        exit(1);
    }
}

int main(int argc, char *argv[]) {
    test_build_simple();
    return 0;
}
