#!/usr/bin/env python3
"""Run the generated C numeric API against pinned vectors with ASan and UBSan."""
import json
import os
from pathlib import Path
import subprocess
import tempfile

crate = Path(__file__).resolve().parent
build = subprocess.run(["cargo", "test", "--locked", "--no-run", "--message-format=json"], cwd=crate, check=True, text=True, stdout=subprocess.PIPE)
outputs = [json.loads(line) for line in build.stdout.splitlines() if line.startswith("{")]
out = next(Path(item["out_dir"]) for item in outputs if item.get("reason") == "build-script-executed" and "molecule-integer-tests" in item["package_id"])
specs = [(bits, signed, order) for bits in [8,16,32,64,128,256] for signed in [False,True] if not (bits == 256 and signed) for order in ["little","big"]]
calls = []
for case in json.loads((crate / "fixtures/vectors.json").read_text()):
    tag = specs.index((case["bits"], case["signed"], case["byte_order"]))
    value = int(case["value"]) % (1 << 256)
    limbs = ",".join(f"{(value >> (64*i)) & ((1 << 64)-1)}ULL" for i in range(4))
    data = ",".join(str(byte) for byte in bytes.fromhex(case["hex"]))
    calls.append(f"{{ const uint8_t data[] = {{{data}}}; const uint64_t limbs[] = {{{limbs}}}; check({tag}, data, sizeof(data), limbs); }}")
driver = r'''
#include <assert.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
/* Exercise the integer header with the limits provided by ckb-c-stdlib. */
#undef INT64_C
#undef UINT64_C
#include "bridge.c"
static void check(unsigned int tag, const uint8_t *bytes, unsigned int width, const uint64_t *value) {
    uint8_t input[40], output[40];
    for (unsigned int offset=0;offset<8;offset++) {
        uint8_t *dst = output + offset;
        memset(input,0xa5,sizeof(input)); memcpy(input+offset,bytes,width);
        memset(output,0xa5,sizeof(output));
        assert(check_c(tag,input+offset,width,value,dst,width)==0);
        assert(memcmp(bytes,dst,width)==0 && dst[width]==0xa5);
        if (offset) assert(output[offset-1]==0xa5);
        for (unsigned int invalid=width-1;invalid<=width+1;invalid+=2) {
            memset(output,0xa5,sizeof(output));
            assert(check_c(tag,input+offset,invalid,value,dst,width)!=0);
            assert(check_c(tag,input+offset,width,value,dst,invalid)!=0);
            for (unsigned int j=0;j<sizeof(output);j++) assert(output[j]==0xa5);
        }
    }
}
int main(void) {
'''+"\n".join(calls)+r'''
    uint8_t charge_bytes[9] = {7,0,0,0,0,0,0,1,0};
    mol_seg_t charge = {charge_bytes, sizeof charge_bytes};
    mol_seg_t amount = MolReader_Charge_get_amount(&charge);
    uint64_t index = 0;
    assert(MolReader_Uint64_read_value(&amount, &index) == MOL_OK && index == 256);
    mol_uint256_t a=mol_uint256_from_u64(255), b=mol_uint256_from_u64(256);
    assert(mol_uint256_compare(a,b)<0);
    mol_uint128_t u128_a={{1, 2}}, u128_b={{2, 2}}, u128_c={{0, 3}};
    assert(mol_uint128_compare(u128_a, u128_a)==0);
    assert(mol_uint128_compare(u128_a, u128_b)<0);
    assert(mol_uint128_compare(u128_c, u128_b)>0);
    assert(mol_uint256_compare(mol_uint256_from_u128(u128_a), mol_uint256_from_u128(u128_b))<0);
    mol_int128_t i128_min={{0, (uint64_t)1 << 63}};
    mol_int128_t i128_neg1={{UINT64_MAX, UINT64_MAX}};
    mol_int128_t i128_zero={{0, 0}};
    mol_int128_t i128_max={{UINT64_MAX, ((uint64_t)1 << 63) - 1}};
    assert(mol_int128_compare(i128_min, i128_neg1)<0);
    assert(mol_int128_compare(i128_neg1, i128_zero)<0);
    assert(mol_int128_compare(i128_zero, i128_max)<0);
    assert(mol_int128_compare(i128_max, i128_min)>0);
    assert(mol_int128_compare(i128_neg1, i128_neg1)==0);
    uint64_t narrowed=123;
    assert(mol_uint256_to_u64(a,&narrowed)==0 && narrowed==255);
    b.limbs[1]=1;
    assert(mol_uint256_to_u64(b,&narrowed)!=0 && narrowed==255);
    mol_uint128_t wide={{123,456}};
    b.limbs[2]=1;
    assert(mol_uint256_to_u128(b,&wide)!=0 && wide.limbs[0]==123 && wide.limbs[1]==456);
    mol_uint256_t value = {{1, 2, 3, 4}};
    mol_seg_t wire = {(uint8_t *)value.limbs, sizeof value.limbs};
    assert(mol_integer_write_limbs(value.limbs, 4, false, &wire) == MOL_OK);
    for (unsigned i = 0; i < 4; ++i) assert(wire.ptr[i * 8 + 7] == 4 - i);
    assert(MolReader_U256Be_read_value(&wire, NULL) == MOL_ERR_DATA);
    assert(MolReader_U256Be_read_value(&wire, &value) == MOL_OK);
    --wire.size;
    assert(MolReader_U256Be_read_value(&wire, &value) == MOL_ERR_TOTAL_SIZE);
    for (unsigned i = 0; i < 4; ++i) assert(value.limbs[i] == i + 1);
    uint8_t guard = 0xa5;
    assert(narrow_i8_c(128, &guard) == MOL_ERR_DATA && guard == 0xa5);
    assert(narrow_i8_c(-129, &guard) == MOL_ERR_DATA && guard == 0xa5);
    assert(narrow_i8_c(INT64_MIN, &guard) == MOL_ERR_DATA && guard == 0xa5);
    assert(narrow_u8_c(256, &guard) == MOL_ERR_DATA && guard == 0xa5);
    assert(narrow_u8_c(UINT64_MAX, &guard) == MOL_ERR_DATA && guard == 0xa5);
    /* Truncated table headers must fail before reading past the 8-byte buffer. */
    uint8_t *short_table = (uint8_t *)malloc(8);
    assert(short_table != NULL);
    const uint32_t bad_offsets[] = {12u, 256u, 0xfffffffcu};
    for (size_t i = 0; i < sizeof(bad_offsets) / sizeof(bad_offsets[0]); ++i) {
        uint32_t total = 8u, off = bad_offsets[i];
        mol_pack_number(short_table, &total);
        mol_pack_number(short_table + 4, &off);
        mol_seg_t seg = {short_table, 8};
        assert(MolReader_Invoice_verify(&seg, true) == MOL_ERR_HEADER);
        if (off == 12u) {
            assert(MolReader_Invoice_verify(&seg, false) == MOL_ERR_HEADER);
        }
    }
    free(short_table);
    /* Compatible verification checks extra trailing field offsets without VLAs. */
    uint8_t extra_table[16 + 8 + 32 + 1];
    memset(extra_table, 0, sizeof(extra_table));
    uint32_t extra_hdr[4] = {sizeof(extra_table), 16u, 24u, 56u};
    for (size_t i = 0; i < 4; ++i) mol_pack_number(extra_table + i * 4, &extra_hdr[i]);
    mol_seg_t extra_seg = {extra_table, sizeof(extra_table)};
    assert(MolReader_Invoice_verify(&extra_seg, false) == MOL_ERR_FIELD_COUNT);
    assert(MolReader_Invoice_verify(&extra_seg, true) == MOL_OK);
    uint32_t bad_extra = sizeof(extra_table) + 1u;
    mol_pack_number(extra_table + 12, &bad_extra);
    assert(MolReader_Invoice_verify(&extra_seg, true) == MOL_ERR_OFFSET);
    return 0;
}
'''
with tempfile.TemporaryDirectory(prefix="molecule-integer-sanitize-") as directory:
    source = Path(directory) / "driver.c"
    proto_check = Path(directory) / "prototypes.c"
    executable = Path(directory) / "test"
    source.write_text(driver)
    # Verify generated headers compile cleanly without MOLECULE_API_DECORATOR
    # under strict warning flags including -Wsign-compare and -Wmissing-prototypes.
    # Treat legacy runtime headers as system includes here because molecule_reader.h
    # and molecule_builder.h predate prototype declarations.
    proto_check.write_text('#include "c/numbers.h"\n')
    cc = os.environ.get("CC", "clang")
    include_dir = str(crate.parent.parent / "bindings/c/include")
    for compiler, dialect, language in [(cc, "c99", "c"), (os.environ.get("CXX", "clang++"), "c++17", "c++")]:
        positive = '''#include "legacy/shared.h"
#include "c/shared.h"
#include "packed/shared.h"
#include "additional/shared.h"
int main(void) {
    uint8_t bytes[] = {1, 2};
    mol_seg_t pair = {bytes, sizeof(bytes)};
    mol_seg_t amount = MolReader_Pair_get_amount(&pair);
    uint16_t value = 0;
    return MolReader_Amount_read_value(&amount, &value) != MOL_OK || value != 258;
}
'''
        subprocess.run([compiler, f"-std={dialect}", "-pedantic-errors", "-I", str(out), "-isystem", include_dir, "-x", language,
                        "-", "-o", str(executable)], input=positive, text=True, check=True)
        subprocess.run([str(executable)], check=True)
        for first, second, diagnostic in [
            ("legacy", "wide", "MolBuilder_Amount_set_nth2"),
            ("swapped", "c", "MolDefault_Amount"),
            ("wrapped", "c", "MolDefault_Amount"),
            ("aggregate", "c", "cannot verify shared Molecule layout Pair"),
            ("c", "legacy", "MolDefault_Amount"),
        ]:
            includes = f'#include "{first}/shared.h"\n#include "{second}/shared.h"\n'
            conflict = subprocess.run(
                [compiler, f"-std={dialect}", "-fsyntax-only", "-I", str(out), "-I", include_dir, "-x", language, "-"],
                input=includes, text=True, capture_output=True,
            )
            assert conflict.returncode and diagnostic in conflict.stderr, (first, second, conflict.stderr)
    strict = [cc, "-std=c11", "-pedantic-errors", "-Wall", "-Wextra", "-Wsign-compare", "-Werror", "-O2", "-I", str(out)]
    subprocess.run(strict + ["-Wmissing-prototypes", "-isystem", include_dir, "-c", str(proto_check), "-o", str(Path(directory) / "prototypes.o")], check=True)
    subprocess.run(strict + ["-I", include_dir, "-Wno-unused-function", "-fsanitize=address,undefined", "-fno-sanitize-recover=all", "-fno-omit-frame-pointer", "-g", str(source), "-o", str(executable)], check=True)
    subprocess.run([str(executable)], check=True)
print(f"C sanitizers: {len(calls)} fixed vectors, 8 alignments, exact length and capacity failures passed")
