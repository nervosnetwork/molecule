## Molecule Schema Language

Molecule data are strongly-typed and not self-describing.

### Grammar

- [EBNF Version](grammar/grammar.ebnf)

### Language Reference

#### Comments

- Line Comments:

```molecule
// This is a line comment
```

- Block Comments:

```molecule
/* This
   is
   a
   block
   comment
 */
```

#### Built-in Types

##### Primitive Type

There is only one built-in primitive type: `byte`.

NOTE:

- The Molecule serialization don't care about the order of user data in which
  a sequence of bytes is stored in a computer's memory.

  You have to pack them in your own way and unpack them by yourself.

##### Composite Types

- `array`

An `array` has an item type and an unsigned integer.

```molecule
array ArrayName [ItemType; N];  // N is an unsigned integer
```

- `struct`

A `struct` has a set of named and typed fields.

```molecule
struct StructName {
    field_name_1: FieldType1,
    field_name_2: FieldType2,
    field_name_3: FieldType3,
}
```

- `vector`

A `vector` has only an item type.

```molecule
vector VectorName <ItemType>;
```

- `table`

A `table` has a set of named and typed fields, same as `struct`.

```molecule
table TableName {
    field_name_1: FieldType1,
    field_name_2: FieldType2,
    field_name_3: FieldType3,
}
```

- `option`

An `option` has only an item type.

```molecule
option OptionName (ItemType);
```

- `union`

A `union` has a set of item types.

```molecule
union UnionName {
    ItemType1,
    ItemType2,
    ItemType3,
}
```

#### Keywords

- `import`

Import types from other schema files.

```molecule
import ../library/common_types;
```
