# Explicit integer semantics

Molecule still stores integers as fixed byte arrays. This opt-in extension assigns
numeric meaning to an existing declaration without changing its wire layout,
default bytes, verifier, Reader/Entity identity, or existing conversion traits.
Unannotated arrays (including `Byte32`) remain opaque. Names never imply an integer.

Supported values are unsigned 8/16/32/64/128/256 bits and signed 8/16/32/64/128 bits,
with explicit little or big endian order. Signed values use two's complement.
Only a direct `array T [byte; N]` with the exact width may be annotated.

## Enable the extension

New schemas can declare a named integer directly.

```molecule
integer Amount = u64be;
```

Use `u` or `i`, the width in bits and an `le` or `be` suffix. Single-byte
`u8` and `i8` also accept the suffix-free spelling, equivalent to little endian.
The compiler lowers the declaration to a byte array and the same integer
specification used by metadata. Imports carry this meaning automatically.

For a deployed `array Amount [byte; 8];` in `amount.mol`, supply an owner sidecar
pinned to its unchanged source. Migrating that declaration to native syntax
requires removing its sidecar entry.

```json
{
  "metadata_version": 1,
  "schema": "amount.mol",
  "schema_sha256": "<SHA256 of the exact schema file bytes>",
  "integers": [
    {"type":"Amount","bits":64,"signed":false,"byte_order":"big"}
  ]
}
```

Relative paths are resolved from the sidecar's directory. Unknown/duplicate fields, unsupported versions,
missing properties, stale pins, duplicate annotations, invalid widths and targets
are errors. An imported type may only be annotated by a sidecar naming its owner
file; a consumer cannot override its meaning. Repeat `--type-metadata` for imports.

```sh
moleculec --schema-file amount.mol --type-metadata amount.json --language rust
```

In a build script, enable `molecule-codegen`'s `schema-extensions` feature and call
`Compiler::type_metadata_file` before `generate_code(...).run()`. `parse_ast()`
validates inputs without producing output. Directory output buffers its result
and atomically replaces a writable destination file after successful validation.
For a destination with multiple hard links, identical output is a no-op and
changed output returns `E_OUTPUT` with every link intact.
Unix replacement preserves the owner, group and mode. On macOS, Linux,
FreeBSD and NetBSD it also preserves ACLs and visible extended attributes.
If inspection or restoration fails, the original stays intact. Privileged
attributes hidden from the caller (Linux `trusted.*` or BSD `system.*`) may
be lost when a changed file is replaced. Linux rejects changes to files with
capability or integrity attributes (`security.capability`, `security.ima`,
`security.evm`). On OpenBSD and DragonFly, changed files on a filesystem with
unknown metadata support return an error. An identical writable file needs
no replacement.
All files are read once into a source snapshot shared by provenance and the AST.
Imported owners use canonical paths relative to the root schema's directory when
representable and otherwise keep the import spelling. Hard links share the
smallest reachable name. Imports resolve from the original spelling's parent
directory; if a file is imported through aliases in multiple directories, every
relative import must resolve to the exact same target. The graph distinguishes
cycles from diamonds.
Graph traversal keeps a constant number of file handles (using inode identity on
Unix and path comparison elsewhere).

## Rust and lazy Rust

Generated ordinary Rust adds `Amount::from_u64`, `Amount::try_to_u64`,
`AmountReader::try_to_u64`, and `AmountReader::try_as_wire_bytes`.
The borrowed wire view is `&[u8; 8]`, checked for exact length; checked getters
return `Result<_, IntegerError>` so `new_unchecked` inputs never panic. Numeric
decoding works at every alignment and on either host byte order. It never casts
wire memory to a native scalar reference. Existing downstream `From<u64>`/Pack
implementations remain available because the generator adds named methods, not
overlapping traits. As before, an owned Entity owns its bytes; its constructor
can allocate.

An ordinary integer vector accepts native values directly. For example,
`Uint64Vec::try_from_values(&cycles)?` writes the count and each scalar in its
declared byte order into one owned buffer. A size exceeding the wire or address
limit returns `IntegerError::Overflow` before allocation.
For application records such as CKB fees, `Uint64Vec::try_from_values_by(&fees,
|fee| fee.as_u64())?` selects the native value as it writes each item. The slice
supplies the count and the schema supplies the byte order.

For an option of an integer, `from_value(Option<T>)` constructs the owned Entity
and `try_value()` on both Entity and Reader returns `Result<Option<T>, IntegerError>`.
Empty, present zero and malformed payloads remain distinguishable. For a vector
of integers, `try_values()?` on both Entity and Reader checks the header and
length once (returning `IntegerError`), then yields native values from borrowed
fixed-width chunks (`ExactSizeIterator + DoubleEndedIterator`). Reading a prefix
requires no collection or full payload scan.

Widths through 128 use Rust scalars. `molecule::integer::Uint256` holds 32 bytes,
has `ZERO`/`MAX` constants, numeric `Ord`, `const fn` LE/BE byte and `u64`/`u128`
conversions (`From`/`TryFrom`), hex formatting, and checked narrowing to u128/u64.
It is a value type, not a native memory layout or arithmetic library. Both
`IntegerError` and `AccessError` are `#[non_exhaustive]`.

The separate lazy backend adds the same `try_to_*` decoding method, using the
shared bounded Cursor reader (reading only the exact requested header and field
ranges without speculatively touching trailing fields).
Short reads are retried; a missing byte, excess logical length or invalid offset
returns an error, and scalar width mismatches consistently return
`IntegerError::Size`. Struct/table integer fields (and optional integer table
fields) add `field_value()`. Integer options add `try_value()`, and integer
fixed vectors and arrays add `get_value(index)`. Existing numeric getters keep
their Rust return types and honor the declared byte order.
Unannotated schemas keep their existing conversions. Lazy generation rejects a
conflict between a conventional scalar name and an explicitly declared width or
signedness. Getter collisions are rejected.
Table numeric access validates the complete offset header in compatible mode
(allowing extra trailing fields) and reads the selected field; it does not
promise recursive validation of other fields.

## C

Generated headers include `molecule_integer.inc` and add prototypes and definitions
for `MolReader_Amount_read_value` and `MolBuilder_Amount_write_value`. Unlike
heap-based `mol_builder_t` builders, `write_value` writes directly into a
caller-provided `mol_seg_t` whose size equals the declared width and whose buffer
covers that many bytes.
The wire buffer may be unaligned. A reader's result pointer must refer to a
writable object aligned for its native type. Null pointers, size mismatches
and out-of-range scalar inputs return an error before changing the destination.

Values through 64 bits use standard C integer types. Wider values use
`mol_uint128_t`, `mol_int128_t`, and `mol_uint256_t`, with least-significant limbs
first (`mol_uint128_compare`, `mol_int128_compare`, `mol_uint256_compare`,
`mol_uint256_from_u64`/`u128`, and checked narrowing to u128/u64). Signed 128
uses two's complement limbs. Use the accessors to encode these structs in schema
byte order. They require no compiler-specific `__int128`. The numeric runtime
include uses a separate filename extension and include guard so generated schema
headers can use the name `molecule_integer.h`, including through imports.
Current plain and annotated headers for the same layout work in either order.
When two headers share declarations and one comes from 0.9.2, include the old
header first. Its byte arrays can supply the wire API when their size and byte
offsets match. A conflict fails C or C++ compilation. Overlapping aggregate
declarations require regenerating the old header because it carries no layout
identity. A current header never reserves an old header's basename guard.

## IR and plugins

Plain schema exports and legacy plugins keep IR v1. For plain schemas with imports,
the CLI sets `MOLECULEC_IR_VERSION=2` on the `--format` probe. A plugin can
return its capability manifest to receive declaration owners in IR v2. A JSON
or YAML format name selects v1 as before. Both bundled Rust generators use
owner paths to distinguish declarations that generate the same Rust name.
V1 exports record the shortest import depth. A detached v1 layout has no
module paths for indirect declarations. If Rust needs to name one, generate
from the schema or supply IR v2. Archived 0.9.2 v1 files store depth 1 for every import. For example, a
type defined by a direct import and one defined by its own import can produce
identical v1 IR. The Rust plugins cannot distinguish these cases from the
archived file alone. Regenerate from the original schemas to recover the
owners and depths in v2.
Extended inputs emit IR v2 with unchanged `wire_ir`, complete source/owner/import
provenance, `logical_types`, and `required_features: ["logical-integer-v1"]`.
JSON/YAML recovery revalidates widths, ownership, layout and import correspondence.
Use `--language - --format json` to inspect it. `--wire-only` explicitly exports
v1 wire structure and warns that integer semantics have been removed.

Keep an accepted IR v2 JSON file per schema root. Pass it to
`--check-integer-contract PUBLISHED_V2_JSON` when generating bindings, or call
`Compiler::integer_contract_file` with the `compiler-plugin` feature. The check
compares the root name and every reachable integer's name, owner, signedness,
width and byte order (treating 8-bit `little` and `big` as equivalent), before
opening output or starting a plugin. It tolerates declaration order, source edits
and migration between equivalent sidecar and native declarations. Compare
`wire_ir` separately for general layout changes.

Before sending extended IR, the CLI asks the explicitly selected local plugin for
`--capabilities`. Protocol 1 advertises `ir_versions`, `formats`, `features`,
`api_flavor`, and exact `integers` specifications. A plugin without matching
capabilities is rejected; there is no silent byte-array fallback. The bundled
Rust, lazy Rust and C plugins support all 22 specifications.

Capability negotiation is bounded to 5 seconds and 64 KiB. Generation defaults
to 60 seconds (`--plugin-timeout`), 64 MiB stdout and 64 KiB stderr. Input/output
are drained concurrently; failed generation is never published. The compiler
uses process-wrap 8.2.1 (MSRV 1.77) to clean up Unix process groups and Windows
Job Objects, including ordinary descendants holding pipes. Worker cleanup gets
an additional bounded 250 ms. This is cancellation for trusted compiler plugins,
not an execution sandbox: a Unix program deliberately escaping its group can
outlive it. The dependency is confined to the host compiler; the Rust and C
runtimes gain no dependencies or unsafe code.

## Validation

`examples/integer-tests` uses 486 pinned valid vectors, covering all widths,
signedness and orders. It compares numeric meaning and fixed wire bytes through
Rust, lazy Rust, C and the published 0.9.2 wire API; it also checks alignments 0–7,
N−1/N/N+1 lengths, 1/3-byte source reads, range failures, CKB fixtures/hashes,
existing conversion traits and an opaque-array compile failure. Every committed
vector is exercised.

`python3 examples/integer-tests/sanitize.py` runs the C cases with ASan and UBSan.
Metadata/IR and real CLI plugin tests live under the corresponding tools crates.
The [integer workflow](../.github/workflows/integers.yaml) tests the compiler,
runs the fixtures with std and bytes_vec, checks the C API with sanitizers,
and builds the no_std bindings for RISC-V.

The legacy `blockchain.mol` fixture comes from CKB commit
`95fd03933ce2a396b84aed12caaa0f984165cd6c` (MIT).

The three CKB schemas under `examples/integer-tests/schemas` are taken from
CKB commit `0b0b3e50d8c6fb0a5dab1b79ffcc569d38be14cd`. Seven fixed arrays
have native integer declarations. The external Rust test uses each schema's
public names, including the discovery port, repeated fees and ordered keys.
