use super::builder::{impl_as_builder_for_struct_or_table, impl_as_builder_for_vector, GenBuilder};
use super::union::GenUnion;
use crate::ast::verified::{self as ast, DefaultContent, HasName};

use case::CaseExt;
use std::io;

pub(super) trait Generator: HasName + DefaultContent {
    fn generate<W: io::Write>(&self, writer: &mut W) -> io::Result<()>;
    fn common_generate<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        let struct_name = self.name().to_camel();

        let define = format!(
            r#"
type {struct_name} struct {{
    inner []byte
}}
        "#,
            struct_name = struct_name
        );
        writeln!(writer, "{}", define)?;

        let impl_ = format!(
            r#"
func {struct_name}FromSliceUnchecked(slice []byte) *{struct_name} {{
    return &{struct_name}{{inner: slice}}
}}
func (s *{struct_name}) AsSlice() []byte {{
    return s.inner
}}
            "#,
            struct_name = struct_name
        );
        writeln!(writer, "{}", impl_)?;

        let default_content = self
            .default_content()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
            .join(",");

        let default = format!(
            r#"
func {struct_name}Default() {struct_name} {{
    return *{struct_name}FromSliceUnchecked([]byte{{ {default_content} }})
}}
            "#,
            struct_name = struct_name,
            default_content = default_content
        );
        writeln!(writer, "{}", default)?;
        Ok(())
    }
}

impl Generator for ast::Option_ {
    fn generate<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(writer, "{}", self.gen_builder())?;
        self.common_generate(writer)?;

        let struct_name = self.name().to_camel();
        let inner = self.typ.name().to_camel();

        let constructor = format!(
            r#"
func {struct_name}FromSlice(slice []byte, compatible bool) (*{struct_name}, error) {{
    if len(slice) == 0 {{
        return &{struct_name}{{inner: slice}}, nil
    }}

    _, err := {inner_type}FromSlice(slice, compatible)
    if err != nil {{
        return nil, err
    }}
    return &{struct_name}{{inner: slice}}, nil
}}
            "#,
            struct_name = struct_name,
            inner_type = inner
        );
        writeln!(writer, "{}", constructor)?;

        let impl_ = format!(
            r#"
func (s *{struct_name}) isSome() bool {{
    return len(s.inner) != 0
}}
func (s *{struct_name}) isNone() bool {{
    return len(s.inner) == 0
}}
func (s *{struct_name}) AsBuilder() {struct_name}Builder {{
    var ret = New{struct_name}Builder()
    if s.isSome() {{
        ret.Set(*{inner_type}FromSliceUnchecked(s.AsSlice()))
    }}
    return *ret
}}
            "#,
            struct_name = struct_name,
            inner_type = inner
        );
        writeln!(writer, "{}", impl_)?;
        Ok(())
    }
}

impl Generator for ast::Union {
    fn generate<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(writer, "{}", self.gen_builder())?;
        self.common_generate(writer)?;
        let struct_name = self.name().to_camel();

        let (union_impl, from_slice_switch_iml) = self.gen_union();
        writeln!(writer, "{}", union_impl)?;

        let struct_constructor = format!(
            r#"
func {struct_name}FromSlice(slice []byte, compatible bool) (*{struct_name}, error) {{
    sliceLen := len(slice)
    if uint32(sliceLen) < HeaderSizeUint {{
        errMsg := strings.Join([]string{{"HeaderIsBroken", "{struct_name}", strconv.Itoa(int(sliceLen)), "<", strconv.Itoa(int(HeaderSizeUint))}}, " ")
        return nil, errors.New(errMsg)
    }}
    itemID := unpackNumber(slice)
    innerSlice := slice[HeaderSizeUint:]

    switch itemID {{
    {from_slice_switch_iml}
    default:
        return nil, errors.New("UnknownItem, {struct_name}")
    }}
    return &{struct_name}{{inner: slice}}, nil
}}
            "#,
            struct_name = struct_name,
            from_slice_switch_iml = from_slice_switch_iml
        );
        writeln!(writer, "{}", struct_constructor)?;

        let struct_impl = format!(
            r#"
func (s *{struct_name}) ItemID() Number {{
    return unpackNumber(s.inner)
}}
func (s *{struct_name}) AsBuilder() {struct_name}Builder {{
    return *New{struct_name}Builder().Set(*s.ToUnion())
}}
            "#,
            struct_name = struct_name
        );
        writeln!(writer, "{}", struct_impl)?;
        Ok(())
    }
}

impl Generator for ast::Array {
    fn generate<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        let struct_name = self.name().to_camel();
        let inner = self.typ.name().to_camel();
        let item_count = self.item_count;
        let total_size = self.total_size();

        writeln!(writer, "{}", self.gen_builder())?;
        self.common_generate(writer)?;

        let impl_ = format!(
            r#"
func {struct_name}FromSlice(slice []byte, _compatible bool) (*{struct_name}, error) {{
    sliceLen := len(slice)
    if sliceLen != {total_size} {{
        errMsg := strings.Join([]string{{"TotalSizeNotMatch", "{struct_name}", strconv.Itoa(int(sliceLen)), "!=", strconv.Itoa({total_size})}}, " ")
        return nil, errors.New(errMsg)
    }}
    return &{struct_name}{{inner: slice}}, nil
}}
        "#,
            struct_name = struct_name,
            total_size = total_size
        );
        writeln!(writer, "{}", impl_)?;

        if self.typ.is_atom() {
            writeln!(
                writer,
                r#"
func (s *{struct_name}) RawData() []byte {{
    return s.inner
}}
            "#,
                struct_name = struct_name
            )?
        }

        for i in 0..self.item_count {
            let func_name = format!("Nth{}", i);
            let start = self.item_size * i;
            let end = self.item_size * (i + 1);

            writeln!(
                writer,
                r#"
func (s *{struct_name}) {func_name}() *{inner_type} {{
    ret := {inner_type}FromSliceUnchecked(s.inner[{start}:{end}])
    return ret
}}
            "#,
                struct_name = struct_name,
                func_name = func_name,
                inner_type = inner,
                start = start,
                end = end
            )?
        }

        let as_builder_internal = (0..item_count)
            .map(|index| format!("t.Nth{index}(*s.Nth{index}())", index = index))
            .collect::<Vec<String>>()
            .join("\n");

        let as_builder = format!(
            r#"
func (s *{struct_name}) AsBuilder() {struct_name}Builder {{
	t := New{struct_name}Builder()
	{as_builder_internal}
	return *t
}}
        "#,
            struct_name = struct_name,
            as_builder_internal = as_builder_internal
        );

        writeln!(writer, "{}", as_builder)?;
        Ok(())
    }
}

impl Generator for ast::Struct {
    fn generate<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        let struct_name = self.name().to_camel();
        let total_size = self.total_size();

        writeln!(writer, "{}", self.gen_builder())?;
        self.common_generate(writer)?;

        let impl_ = format!(
            r#"
func {struct_name}FromSlice(slice []byte, _compatible bool) (*{struct_name}, error) {{
    sliceLen := len(slice)
    if sliceLen != {total_size} {{
        errMsg := strings.Join([]string{{"TotalSizeNotMatch", "{struct_name}", strconv.Itoa(int(sliceLen)), "!=", strconv.Itoa({total_size})}}, " ")
        return nil, errors.New(errMsg)
    }}
    return &{struct_name}{{inner: slice}}, nil
}}
        "#,
            struct_name = struct_name,
            total_size = total_size
        );
        writeln!(writer, "{}", impl_)?;

        let (_, each_getter) = self.inner.iter().zip(self.field_size.iter()).fold(
            (0, Vec::with_capacity(self.inner.len())),
            |(mut offset, mut getters), (f, s)| {
                let func_name = f.name.to_camel();
                let inner = f.typ.name().to_camel();

                let start = offset;
                offset += s;
                let end = offset;
                let getter = format!(
                    r#"
func (s *{struct_name}) {func_name}() *{inner} {{
    ret := {inner}FromSliceUnchecked(s.inner[{start}:{end}])
    return ret
}}
                "#,
                    struct_name = struct_name,
                    inner = inner,
                    start = start,
                    end = end,
                    func_name = func_name
                );

                getters.push(getter);
                (offset, getters)
            },
        );

        writeln!(writer, "{}", each_getter.join("\n"))?;

        let as_builder = impl_as_builder_for_struct_or_table(&struct_name, &self.inner[..]);
        writeln!(writer, "{}", as_builder)?;

        Ok(())
    }
}

impl Generator for ast::FixVec {
    fn generate<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        let struct_name = self.name().to_camel();
        let inner = self.typ.name().to_camel();
        let item_size = self.item_size;

        writeln!(writer, "{}", self.gen_builder())?;
        self.common_generate(writer)?;

        let constructor = format!(
            r#"
func {struct_name}FromSlice(slice []byte, _compatible bool) (*{struct_name}, error) {{
    sliceLen := len(slice)
    if sliceLen < int(HeaderSizeUint) {{
        errMsg := strings.Join([]string{{"HeaderIsBroken", "{struct_name}", strconv.Itoa(int(sliceLen)), "<", strconv.Itoa(int(HeaderSizeUint))}}, " ")
        return nil, errors.New(errMsg)
    }}
    itemCount := unpackNumber(slice)
    if itemCount == 0 {{
        if sliceLen != int(HeaderSizeUint) {{
            errMsg := strings.Join([]string{{"TotalSizeNotMatch", "{struct_name}", strconv.Itoa(int(sliceLen)), "!=", strconv.Itoa(int(HeaderSizeUint))}}, " ")
            return nil, errors.New(errMsg)
        }}
        return &{struct_name}{{inner: slice}}, nil
    }}
    totalSize := int(HeaderSizeUint) + int({item_size}*itemCount)
    if sliceLen != totalSize {{
        errMsg := strings.Join([]string{{"TotalSizeNotMatch", "{struct_name}", strconv.Itoa(int(sliceLen)), "!=", strconv.Itoa(int(totalSize))}}, " ")
        return nil, errors.New(errMsg)
    }}
    return &{struct_name}{{inner: slice}}, nil
}}
            "#,
            struct_name = struct_name,
            item_size = item_size
        );
        writeln!(writer, "{}", constructor)?;

        let impl_ = format!(
            r#"
func (s *{struct_name}) TotalSize() uint {{
    return uint(HeaderSizeUint) * (s.ItemCount()+1)
}}
func (s *{struct_name}) ItemCount() uint {{
    number := uint(unpackNumber(s.inner))
    return number
}}
func (s *{struct_name}) Len() uint {{
    return s.ItemCount()
}}
func (s *{struct_name}) IsEmpty() bool {{
    return s.Len() == 0
}}
// if *{inner_type} is nil, index is out of bounds
func (s *{struct_name}) Get(index uint) *{inner_type} {{
    var re *{inner_type}
    if index < s.Len() {{
        start := uint(HeaderSizeUint) + {item_size}*index
        end := start + {item_size}
        re = {inner_type}FromSliceUnchecked(s.inner[start:end])
    }}
    return re
}}
        "#,
            struct_name = struct_name,
            inner_type = inner,
            item_size = item_size
        );
        writeln!(writer, "{}", impl_)?;

        if self.typ.is_atom() {
            writeln!(
                writer,
                r#"
func (s *{struct_name}) RawData() []byte {{
    return s.inner[HeaderSizeUint:]
}}
            "#,
                struct_name = struct_name
            )?
        }
        let as_builder = impl_as_builder_for_vector(&struct_name);
        writeln!(writer, "{}", as_builder)?;
        Ok(())
    }
}

impl Generator for ast::DynVec {
    fn generate<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        let struct_name = self.name().to_camel();
        let inner = self.typ.name().to_camel();

        writeln!(writer, "{}", self.gen_builder())?;
        self.common_generate(writer)?;

        let constructor = format!(
            r#"
func {struct_name}FromSlice(slice []byte, compatible bool) (*{struct_name}, error) {{
    sliceLen := len(slice)

    if uint32(sliceLen) < HeaderSizeUint {{
        errMsg := strings.Join([]string{{"HeaderIsBroken", "{struct_name}", strconv.Itoa(int(sliceLen)), "<", strconv.Itoa(int(HeaderSizeUint))}}, " ")
        return nil, errors.New(errMsg)
    }}

    totalSize := unpackNumber(slice)
    if Number(sliceLen) != totalSize {{
        errMsg := strings.Join([]string{{"TotalSizeNotMatch", "{struct_name}", strconv.Itoa(int(sliceLen)), "!=", strconv.Itoa(int(totalSize))}}, " ")
        return nil, errors.New(errMsg)
    }}

    if uint32(sliceLen) == HeaderSizeUint {{
        return &{struct_name}{{inner: slice}}, nil
    }}

    if uint32(sliceLen) < HeaderSizeUint*2 {{
        errMsg := strings.Join([]string{{"TotalSizeNotMatch", "{struct_name}", strconv.Itoa(int(sliceLen)), "<", strconv.Itoa(int(HeaderSizeUint*2))}}, " ")
        return nil, errors.New(errMsg)
    }}

    offsetFirst := unpackNumber(slice[HeaderSizeUint:])
    if offsetFirst%4 != 0 || uint32(offsetFirst) < HeaderSizeUint*2 {{
        errMsg := strings.Join([]string{{"OffsetsNotMatch", "{struct_name}", strconv.Itoa(int(offsetFirst%4)), "!= 0", strconv.Itoa(int(offsetFirst)), "<", strconv.Itoa(int(HeaderSizeUint*2))}}, " ")
        return nil, errors.New(errMsg)
    }}

    itemCount := offsetFirst/4 - 1
    headerSize := HeaderSizeUint * (uint32(itemCount) + 1)
    if uint32(sliceLen) < headerSize {{
        errMsg := strings.Join([]string{{"HeaderIsBroken", "{struct_name}", strconv.Itoa(int(sliceLen)), "<", strconv.Itoa(int(headerSize))}}, " ")
        return nil, errors.New(errMsg)
    }}

    offsets := make([]uint32, itemCount)

    for i := 0; i < int(itemCount); i++ {{
        offsets[i] = uint32(unpackNumber(slice[HeaderSizeUint:][4*i:]))
    }}

    offsets = append(offsets, uint32(totalSize))

    for i := 0; i < len(offsets); i++ {{
        if i&1 != 0 && offsets[i-1] > offsets[i] {{
            errMsg := strings.Join([]string{{"OffsetsNotMatch", "{struct_name}"}}, " ")
            return nil, errors.New(errMsg)
        }}
    }}

    for i := 0; i < len(offsets); i++ {{
        if i&1 != 0 {{
            start := offsets[i-1]
            end := offsets[i]
            _, err := {inner_type}FromSlice(slice[start:end], compatible)

            if err != nil {{
                return nil, err
            }}
        }}
    }}

    return &{struct_name}{{inner: slice}}, nil
}}
            "#,
            struct_name = struct_name,
            inner_type = inner
        );
        writeln!(writer, "{}", constructor)?;

        let impl_ = format!(
            r#"
func (s *{struct_name}) TotalSize() uint {{
    return uint(unpackNumber(s.inner))
}}
func (s *{struct_name}) ItemCount() uint {{
    var number uint = 0
    if uint32(s.TotalSize()) == HeaderSizeUint {{
        return number
    }}
    number = uint(unpackNumber(s.inner[HeaderSizeUint:]))/4 - 1
    return number
}}
func (s *{struct_name}) Len() uint {{
    return s.ItemCount()
}}
func (s *{struct_name}) IsEmpty() bool {{
    return s.Len() == 0
}}
func (s *{struct_name}) itemOffsets() [][4]byte {{
    // Preventing index out-of-bounds array accesses when not alignment
    dataSize := len(s.inner[HeaderSizeUint:]) - len(s.inner[HeaderSizeUint:])%4
    cap := dataSize / 4
    ret := make([][4]byte, cap)
    var firstIdx, secondIdex int
    for i := 0; i < dataSize; i++ {{
        firstIdx = i >> 2
        if firstIdx > cap {{
            break
        }}
        secondIdex = i % 4
        ret[firstIdx][secondIdex] = s.inner[HeaderSizeUint:][firstIdx*4:][secondIdex]
    }}
    return ret
}}
// if *{inner_type} is nil, index is out of bounds
func (s *{struct_name}) Get(index uint) *{inner_type} {{
    var b *{inner_type}
    if index < s.Len() {{
        offsets := s.itemOffsets()
        start := unpackNumber(offsets[index][:])

        if index == s.Len()-1 {{
            b = {inner_type}FromSliceUnchecked(s.inner[start:])
        }} else {{
            end := unpackNumber(offsets[index+1][:])
            b = {inner_type}FromSliceUnchecked(s.inner[start:end])
        }}
    }}
    return b
}}
            "#,
            struct_name = struct_name,
            inner_type = inner
        );
        writeln!(writer, "{}", impl_)?;
        let as_builder = impl_as_builder_for_vector(&struct_name);
        writeln!(writer, "{}", as_builder)?;
        Ok(())
    }
}

impl Generator for ast::Table {
    fn generate<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        let field_count = self.inner.len();
        let struct_name = self.name().to_camel();

        writeln!(writer, "{}", self.gen_builder())?;
        self.common_generate(writer)?;

        let constructor = if self.inner.is_empty() {
            format!(
                r#"
func New{struct_name}() {struct_name} {{
    s := new(bytes.Buffer)
    s.Write(packNumber(Number(HeaderSizeUint)))
}}
func {struct_name}FromSlice(slice []byte, compatible bool) (*{struct_name}, error) {{
    sliceLen := len(slice)
    if uint32(sliceLen) < HeaderSizeUint {{
        return nil, errors.New("HeaderIsBroken")
    }}

    totalSize := unpackNumber(slice)
    if Number(sliceLen) != totalSize {{
        return nil, errors.New("TotalSizeNotMatch")
    }}

    if uint32(sliceLen) > HeaderSizeUint && !compatible {{
        return nil, errors.New("FieldCountNotMatch")
    }}
    return &{struct_name}{{inner: slice}}, nil
}}
            "#,
                struct_name = struct_name
            )
        } else {
            let verify_fields = self
                .inner
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    let field = f.typ.name().to_camel();
                    let start = i;
                    let end = i + 1;
                    format!(
                        r#"
_, err := {field}FromSlice(slice[offsets[{start}]:offsets[{end}]], compatible)
if err != nil {{
    return nil, err
}}
                "#,
                        field = field,
                        start = start,
                        end = end
                    )
                })
                .collect::<Vec<String>>()
                .join("\n");

            format!(
                r#"
func {struct_name}FromSlice(slice []byte, compatible bool) (*{struct_name}, error) {{
    sliceLen := len(slice)
    if uint32(sliceLen) < HeaderSizeUint {{
        errMsg := strings.Join([]string{{"HeaderIsBroken", "{struct_name}", strconv.Itoa(int(sliceLen)), "<", strconv.Itoa(int(HeaderSizeUint))}}, " ")
        return nil, errors.New(errMsg)
    }}

    totalSize := unpackNumber(slice)
    if Number(sliceLen) != totalSize {{
        errMsg := strings.Join([]string{{"TotalSizeNotMatch", "{struct_name}", strconv.Itoa(int(sliceLen)), "!=", strconv.Itoa(int(totalSize))}}, " ")
        return nil, errors.New(errMsg)
    }}

    if uint32(sliceLen) == HeaderSizeUint && {field_count} == 0 {{
        return &{struct_name}{{inner: slice}}, nil
    }}

    if uint32(sliceLen) < HeaderSizeUint*2 {{
        errMsg := strings.Join([]string{{"TotalSizeNotMatch", "{struct_name}", strconv.Itoa(int(sliceLen)), "<", strconv.Itoa(int(HeaderSizeUint*2))}}, " ")
        return nil, errors.New(errMsg)
    }}

    offsetFirst := unpackNumber(slice[HeaderSizeUint:])
    if offsetFirst%4 != 0 || uint32(offsetFirst) < HeaderSizeUint*2 {{
        errMsg := strings.Join([]string{{"OffsetsNotMatch", "{struct_name}", strconv.Itoa(int(offsetFirst%4)), "!= 0", strconv.Itoa(int(offsetFirst)), "<", strconv.Itoa(int(HeaderSizeUint*2))}}, " ")
        return nil, errors.New(errMsg)
    }}

    fieldCount := offsetFirst/4 - 1
    if fieldCount < {field_count} {{
        return nil, errors.New("FieldCountNotMatch")
    }} else if !compatible && fieldCount > {field_count} {{
        return nil, errors.New("FieldCountNotMatch")
    }}

    headerSize := HeaderSizeUint * (uint32(fieldCount) + 1)
    if uint32(sliceLen) < headerSize {{
        errMsg := strings.Join([]string{{"HeaderIsBroken", "{struct_name}", strconv.Itoa(int(sliceLen)), "<", strconv.Itoa(int(headerSize))}}, " ")
        return nil, errors.New(errMsg)
    }}

    offsets := make([]uint32, {field_count})

    for i := 0; i < {field_count}; i++ {{
        offsets[i] = uint32(unpackNumber(slice[HeaderSizeUint:][4*i:]))
    }}
    offsets = append(offsets, uint32(totalSize))

    for i := 0; i < len(offsets); i++ {{
        if i&1 != 0 && offsets[i-1] > offsets[i] {{
            return nil, errors.New("OffsetsNotMatch")
        }}
    }}
    {verify_fields}

    return &{struct_name}{{inner: slice}}, nil
}}
            "#,
                struct_name = struct_name,
                field_count = field_count,
                verify_fields = verify_fields
            )
        };
        writeln!(writer, "{}", constructor)?;

        let impl_ = format!(
            r#"
func (s *{struct_name}) TotalSize() uint {{
    return uint(unpackNumber(s.inner))
}}
func (s *{struct_name}) FieldCount() uint {{
    var number uint = 0
    if uint32(s.TotalSize()) == HeaderSizeUint {{
        return number
    }}
    number = uint(unpackNumber(s.inner[HeaderSizeUint:]))/4 - 1
    return number
}}
func (s *{struct_name}) Len() uint {{
    return s.FieldCount()
}}
func (s *{struct_name}) IsEmpty() bool {{
    return s.Len() == 0
}}
func (s *{struct_name}) FieldOffsets() [][4]byte {{
    // Preventing index out-of-bounds array accesses when not alignment
    dataSize := len(s.inner[HeaderSizeUint:]) - len(s.inner[HeaderSizeUint:])%4
    cap := dataSize / 4
    ret := make([][4]byte, cap)
    var firstIdx, secondIdex int
    for i := 0; i < dataSize; i++ {{
        firstIdx = i >> 2
        if firstIdx > cap {{
            break
        }}
        secondIdex = i % 4
        ret[firstIdx][secondIdex] = s.inner[HeaderSizeUint:][firstIdx*4:][secondIdex]
    }}
    return ret
}}

func (s *{struct_name}) CountExtraFields() uint {{
    return s.FieldCount() - {field_count}
}}

func (s *{struct_name}) hasExtraFields() bool {{
    return {field_count} != s.FieldCount()
}}
            "#,
            struct_name = struct_name,
            field_count = field_count,
        );
        writeln!(writer, "{}", impl_)?;

        let (getter_stmt_last, getter_stmt) = {
            let getter_stmt_last = "s.inner[start:]".to_string();
            let getter_stmt = "s.inner[start:end]".to_string();
            (getter_stmt_last, getter_stmt)
        };
        let each_getter = self
            .inner
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let func = f.name.to_camel();

                let inner = f.typ.name().to_camel();
                let start = i;
                let end = i + 1;
                if i == self.inner.len() - 1 {
                    format!(
                        r#"
func (s *{struct_name}) {func}() *{inner} {{
    var ret *{inner}
    offsets := s.FieldOffsets()
    start := unpackNumber(offsets[0][:])
    if s.hasExtraFields() {{
        end := unpackNumber(offsets[1][:])
        ret = {inner}FromSliceUnchecked({getter_stmt})
    }} else {{
        ret = {inner}FromSliceUnchecked({getter_stmt_last})
    }}
    return ret
}}
                        "#,
                        struct_name = struct_name,
                        func = func,
                        inner = inner,
                        getter_stmt = getter_stmt,
                        getter_stmt_last = getter_stmt_last
                    )
                } else {
                    format!(
                        r#"
func (s *{struct_name}) {func}() *{inner} {{
    offsets := s.FieldOffsets()
    start := unpackNumber(offsets[{start}][:])
    end := unpackNumber(offsets[{end}][:])
    {inner}FromSliceUnchecked({getter_stmt})
}}
               "#,
                        struct_name = struct_name,
                        func = func,
                        inner = inner,
                        getter_stmt = getter_stmt,
                        start = start,
                        end = end
                    )
                }
            })
            .collect::<Vec<_>>();
        writeln!(writer, "{}", each_getter.join("\n"))?;

        let as_builder = impl_as_builder_for_struct_or_table(&struct_name, &self.inner[..]);
        writeln!(writer, "{}", as_builder)?;
        Ok(())
    }
}
