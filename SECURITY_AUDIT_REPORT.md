# Molecule Security Audit Report

**Date:** 2026-04-10
**Scope:** C bindings (`bindings/c/include/`) + Code generator (`tools/codegen/src/`)
**Total Findings:** 58 (38 Critical/High, 19 Medium, 6 Low)

---

## Part 1: C Bindings Security Audit (32 findings)

Analyzed files: `molecule_reader.h` (264 lines), `molecule_builder.h` (289 lines) under `bindings/c/include/`.

### CRITICAL (6 findings)

**BUG-1: Integer overflow in `mol_fixvec_verify` — verification bypass**
- File: `bindings/c/include/molecule_reader.h:122`
- `mol_num_t total_size = MOL_NUM_T_SIZE + item_size * item_count;` — `item_size * item_count` can silently wrap uint32, causing verification to pass on malformed data.

**BUG-2: Integer overflow in `mol_fixvec_slice_by_index` — OOB read**
- File: `bindings/c/include/molecule_reader.h:190`
- `item_size * item_index` can overflow uint32 in the pointer offset calculation.

**BUG-3: Integer overflow in `mol_union_builder_set` — heap overflow**
- File: `bindings/c/include/molecule_builder.h:149`
- `MOL_NUM_T_SIZE + data_len` wraps when `data_len` is near `UINT32_MAX`, causing small allocation + large memcpy.

**BUG-4: Integer overflow in `mol_dynvec_builder_finalize` — heap overflow**
- File: `bindings/c/include/molecule_builder.h:260`
- Sum of three uint32 values overflows, small malloc + large memcpy.

**BUG-5: Integer overflow in `mol_fixvec_builder_finalize` — heap overflow**
- File: `bindings/c/include/molecule_builder.h:247`
- `MOL_NUM_T_SIZE + builder.data_used` wraps.

**BUG-6: Capacity-doubling overflow — infinite loop or heap overflow**
- File: `bindings/c/include/molecule_builder.h:167-225`
- `data_cap *= 2` overflows to 0 when `data_cap >= 0x80000000`, causing infinite loop or zero-size realloc.

### HIGH (13 findings)

**BUG-7 through BUG-11: No NULL checks after malloc/realloc**
- File: `bindings/c/include/molecule_builder.h:77,88,103,106,132,142,152,169,179,189,193,209,225,248,261`
- Every allocation can return NULL under OOM. Immediate dereference causes crash.

**BUG-12: `mol_unpack_number` has no NULL/bounds check**
- File: `bindings/c/include/molecule_reader.h:84`
- Reads 4 bytes unconditionally from src pointer.

**BUG-13: `mol_union_unpack` missing size check**
- File: `bindings/c/include/molecule_reader.h:142-148`
- No check that `input->size >= 4`. Causes OOB read + uint32 underflow on size field.

**BUG-14: `mol_fixvec_length` no bounds check**
- File: `bindings/c/include/molecule_reader.h:151-153`

**BUG-15: `mol_dynvec_length` — OOB read + underflow**
- File: `bindings/c/include/molecule_reader.h:156-162`
- Reads at offset 4 without checking `size >= 8`. Division result 0 causes `(0/4)-1` underflow to `UINT32_MAX`.

**BUG-16: `mol_dynvec_slice_by_index` trusts untrusted offsets**
- File: `bindings/c/include/molecule_reader.h:197-221`
- Offsets read from buffer not validated against `total_size`. Returns segments pointing outside buffer.

**BUG-17: `mol_slice_by_offset` no bounds check**
- File: `bindings/c/include/molecule_reader.h:175-180`

**BUG-18: `mol_fixvec_slice_raw_bytes` trusts encoded length**
- File: `bindings/c/include/molecule_reader.h:231-236`
- Returned segment size from untrusted data, not validated against buffer bounds.

**BUG-19: `mol_table_slice_by_index` discards error, returns uninitialized data**
- File: `bindings/c/include/molecule_reader.h:225-228`
- errno from `dynvec_slice_by_index` is ignored; on error, returns garbage seg.

### MEDIUM (9 findings)

**BUG-20:** No DynVec/Table verification function exists — all DynVec/Table getters are exploitable since no verification rejects malformed data.

**BUG-21:** `mol_fixvec_verify` doesn't handle `item_size=0` (`reader.h:114`) — allows arbitrary `item_count` with 4-byte buffer.

**BUG-22:** `mol_union_builder_initialize` heap overflow when `default_ptr=NULL` (`builder.h:92-93`) — writes at offset 4 without checking allocation >= 5.

**BUG-23:** `mol_builder_set_byte_by_offset` / `set_by_offset` no bounds check (`builder.h:158-163`).

**BUG-24:** `mol_table_builder_add` no bounds check on `field_index` (`builder.h:206-233`).

**BUG-25:** `mol_dynvec_builder_finalize` mutates through value-copy alias (`builder.h:265-267`).

**BUG-26:** `mol_contained_by` has INVERTED boolean semantics (`reader.h:239-247`) — returns `MOL_ERR_OFFSET` (3=true) on failure and `MOL_OK` (0=false) on success. Any safety check using this is backwards.

**BUG-27:** Pointer comparison UB in `mol_contained_by` (`reader.h:240,243`).

**BUG-28:** errno naming collision with POSIX (`reader.h:65`).

### LOW (4 findings)

**BUG-29:** Fragile `is_le()` macro redefinition.
**BUG-30:** `mol_builder_discard` leaves dangling pointers.
**BUG-31:** No sensitive data clearing.
**BUG-32:** ODR violation with default empty `MOLECULE_API_DECORATOR`.

### Top Attack Scenarios (C Bindings)

1. Craft FixVec with overflowing `item_size*item_count` (BUG-1) → verification passes → getter reads arbitrary memory (BUG-2)
2. Supply DynVec with malicious offsets (no verifier, BUG-20) → getters return OOB segments (BUG-16)
3. Trigger builder heap corruption via large `data_len` causing integer overflow (BUG-3/4/5)
4. Any code using `mol_contained_by` as safety check is inverted (BUG-26) — allows unsafe segments, rejects safe ones

---

## Part 2: Codegen Source Code Audit (26 findings)

Analyzed: All 42 source files under `tools/codegen/src/`.

### CRITICAL (3 findings)

**CGEN-1: Generated C Table verifier uses VLA — stack overflow via attacker-controlled `field_count`**
- File: `tools/codegen/src/generator/languages/c/reader.rs:331`
- Generated code: `mol_num_t offsets[field_count+1];`
- `field_count` is derived from untrusted input. A malicious payload with a large first-offset value causes stack overflow. VLAs are optional in C11 and unsupported in C++.

**CGEN-2: Generated C Table builder has no NULL check after malloc**
- File: `tools/codegen/src/generator/languages/c/builder.rs:292`
- Generated code: `res.seg.ptr = (uint8_t*)malloc(res.seg.size);`
- If malloc returns NULL, subsequent writes cause undefined behavior.

**CGEN-3: C header injection via unsanitized import paths**
- File: `tools/codegen/src/generator/languages/c/import.rs:11-18`
- Import paths written directly into `#include` directives without sanitization.

### HIGH (8 findings)

**CGEN-4: Import processing has no recursion depth limit — stack overflow DoS**
- File: `tools/codegen/src/ast/raw/utils.rs:172-201`

**CGEN-5: Parser panics on file errors instead of returning Result**
- Files: `parser/mod.rs:12`, `ast/raw/utils.rs:218-219`, `ast/raw/utils.rs:156`, `ast/raw/utils.rs:187,193`

**CGEN-6: Unbounded integer parsing — panic on overflow**
- Files: `grammar.pest:90`, `grammar.pest:12`, `ast/raw/utils.rs:21`

**CGEN-7: Generated Rust DynVec `get_unchecked` underflows when empty**
- File: `tools/codegen/src/generator/languages/rust/getters.rs:255`

**CGEN-8: Generated Rust Table `count_extra_fields` underflows**
- File: `tools/codegen/src/generator/languages/rust/properties.rs:109`

**CGEN-9: Generated Rust DynVec `item_count()` assumes well-formed offsets**
- File: `tools/codegen/src/generator/languages/rust/properties.rs:76-78`

**CGEN-10: Compiler unwraps throughout — no graceful error handling**
- File: `tools/codegen/src/compiler.rs:130,139-141,146-147,152-154`

**CGEN-11: IR format `generate()` unwraps serialization**
- File: `tools/codegen/src/ir/format.rs:76`

### MEDIUM (10 findings)

**CGEN-12:** C macro definitions use unsanitized type names (`c/utilities.rs:80-84`)
**CGEN-13:** Case-insensitive duplicate check is O(n²) (`ast/verified/complete.rs:180-187`)
**CGEN-14:** Rust Array/Struct verification doesn't verify inner composite items (`rust/reader/implementation.rs:76-88,91-103`)
**CGEN-15:** Lazy reader iterator `next()` unwraps instead of propagating errors (`rust_lazy_reader/generator.rs:477,506,487,514`)
**CGEN-16:** Lazy reader panics for Option nesting level > 2 (`rust_lazy_reader/generator.rs:660`)
**CGEN-17:** Union item ID cast to u32 could silently truncate (`ast/verified/default_content.rs:25`)
**CGEN-18:** Import cycle detection is incomplete for long chains (`ast/raw/utils.rs:187-194`)
**CGEN-19:** `paths.pop().unwrap()` in import parsing (`ast/raw/utils.rs:135`)
**CGEN-20:** Generated Rust Union `to_enum()` panics on unknown item_id (`rust/getters.rs:80`)
**CGEN-21:** DynVec verify loop variable naming collision with C `errno` (`c/reader.rs:241`)

### LOW (5 findings)

**CGEN-22:** No keyword collision validation for generated identifiers
**CGEN-23:** `ast/raw/utils.rs:17` unwraps pest iterator
**CGEN-24:** Grammar requires at least one declaration (`grammar.pest:97`)
**CGEN-25:** Generated C code uses `mol_contained_by` which has inverted semantics (`c/reader.rs:238,253,359`)
**CGEN-26:** `calculate_capacity` uses `next_power_of_two` — panics at overflow (`c/builder.rs:83`)

### Top Risk Scenarios (Codegen)

1. Malicious schema file with deep import chains causes stack overflow (CGEN-4), or with huge numbers causes panic (CGEN-6)
2. Generated C code processes attacker-crafted data: Table verifier stack overflow via VLA (CGEN-1), NULL dereference after failed malloc (CGEN-2)
3. Generated Rust code used with `new_unchecked` on untrusted data: arithmetic underflows (CGEN-7/8/9)
4. Supply-chain attack via crafted schema filenames injecting C headers (CGEN-3)

---

## Overall Summary

| Component | Critical | High | Medium | Low | Total |
|-----------|----------|------|--------|-----|-------|
| C Bindings | 6 | 13 | 9 | 4 | 32 |
| Codegen | 3 | 8 | 10 | 5 | 26 |
| **Total** | **9** | **21** | **19** | **9** | **58** |
