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
    mol_uint256_t a={{255,0,0,0}}, b={{256,0,0,0}};
    assert(mol_uint256_compare(a,b)<0);
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
    return 0;
}
'''
with tempfile.TemporaryDirectory(prefix="molecule-integer-sanitize-") as directory:
    source = Path(directory) / "driver.c"
    executable = Path(directory) / "test"
    source.write_text(driver)
    subprocess.run([os.environ.get("CC","clang"), "-std=c11", "-pedantic-errors", "-O2", "-fsanitize=address,undefined", "-fno-sanitize-recover=all", "-fno-omit-frame-pointer", "-g", "-I", str(out), "-I", str(crate.parent.parent / "bindings/c/include"), str(source), "-o", str(executable)], check=True)
    subprocess.run([str(executable)], check=True)
print(f"C sanitizers: {len(calls)} fixed vectors, 8 alignments, exact length and capacity failures passed")
