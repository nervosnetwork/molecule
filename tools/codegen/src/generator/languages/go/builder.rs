use crate::ast::verified::{self as ast, HasName};

use case::CaseExt;

pub(in super::super) trait GenBuilder {
    fn gen_builder(&self) -> String;
}

impl GenBuilder for ast::Option_ {
    fn gen_builder(&self) -> String {
        let struct_name = self.name().to_camel();
        let inner_type = self.typ.name().to_camel();

        let define = format!(
            r#"
type {struct_name}Builder struct {{
	isNone bool
	inner  {inner_type}
}}
func New{struct_name}Builder() *{struct_name}Builder {{
	return &{struct_name}Builder{{isNone: true, inner: {inner_type}Default()}}
}}
func (s *{struct_name}Builder) Set(v {inner_type}) *{struct_name}Builder {{
	s.isNone = false
	s.inner = v
	return s
}}
func (s *{struct_name}Builder) Build() {struct_name} {{
	var ret {struct_name}
	if s.isNone {{
		ret = {struct_name}{{inner: []byte{{}}}}
	}} else {{
		ret = {struct_name}{{inner: s.inner.AsSlice()}}
	}}
	return ret
}}
            "#,
            struct_name = struct_name,
            inner_type = inner_type
        );

        define
    }
}

impl GenBuilder for ast::Union {
    fn gen_builder(&self) -> String {
        let struct_name = self.name().to_camel();
        let define = format!(
            r#"
type {struct_name}Builder struct {{
	inner  {struct_name}Union
}}
func New{struct_name}Builder() *{struct_name}Builder {{
    v := {struct_name}Default()
	return &{struct_name}Builder{{inner: *v.ToUnion()}}
}}
func (s *{struct_name}Builder) Set(v {struct_name}Union) *{struct_name}Builder {{
	s.inner = v
	return s
}}
func (s *{struct_name}Builder) Build() {struct_name} {{
	b := new(bytes.Buffer)
    b.Write(packNumber(s.inner.itemID))
    b.Write(s.inner.AsSlice())

    return {struct_name}{{inner: b.Bytes()}}
}}
          "#,
            struct_name = struct_name
        );
        define
    }
}

impl GenBuilder for ast::Array {
    fn gen_builder(&self) -> String {
        let struct_name = self.name().to_camel();
        let inner_type = self.typ.name().to_camel();
        let item_count = self.item_count;

        let new_default = (0..item_count)
            .map(|_| format!("{}Default()", inner_type))
            .collect::<Vec<String>>()
            .join(",");

        let define = format!(
            r#"
type {struct_name}Builder struct {{
	inner [{item_count}]{inner_type}
}}

func New{struct_name}Builder() *{struct_name}Builder {{
	return &{struct_name}Builder{{inner: [{item_count}]{inner_type}{{{new_default}}}}}
}}

func (s *{struct_name}Builder) Build() {struct_name} {{
	b := new(bytes.Buffer)
	len := len(s.inner)
	for i := 0; i < len; i++ {{
		b.Write(s.inner[i].AsSlice())
	}}
	return {struct_name}{{inner: b.Bytes()}}
}}
        "#,
            inner_type = inner_type,
            item_count = item_count,
            struct_name = struct_name,
            new_default = new_default
        );

        let entire_setter = format!(
            r#"
func (s *{struct_name}Builder) Set(v [{item_count}]{inner_type}) *{struct_name}Builder {{
	s.inner = v
	return s
}}
        "#,
            struct_name = struct_name,
            inner_type = inner_type,
            item_count = item_count
        );
        let each_setter = (0..item_count)
            .map(|index| {
                format!(
                    r#"
func (s *{struct_name}Builder) Nth{index}(v {inner_type}) *{struct_name}Builder {{
	s.inner[{index}] = v
	return s
}}
                "#,
                    struct_name = struct_name,
                    inner_type = inner_type,
                    index = index
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        vec![define, entire_setter, each_setter].join("\n")
    }
}

impl GenBuilder for ast::Struct {
    fn gen_builder(&self) -> String {
        let struct_name = self.name().to_camel();

        let define = def_builder_for_struct_or_table(&struct_name, &self.inner[..]);
        let setter = impl_setters_for_struct_or_table(&struct_name, &self.inner[..]);
        let default = impl_default_for_struct_or_table(&struct_name, &self.inner[..]);

        let fields_encode = self
            .inner
            .iter()
            .map(|f| {
                let field_name = &f.name;
                format!("b.Write(s.{}.AsSlice())", field_name)
            })
            .collect::<Vec<String>>()
            .join("\n");

        let build = format!(
            r#"
func (s *{struct_name}Builder) Build() {struct_name} {{
    b := new(bytes.Buffer)
    {fields_encode}
    return {struct_name}{{inner: b.Bytes()}}
}}
            "#,
            struct_name = struct_name,
            fields_encode = fields_encode
        );

        vec![define, build, setter, default].join("\n")
    }
}

impl GenBuilder for ast::FixVec {
    fn gen_builder(&self) -> String {
        let struct_name = self.name().to_camel();
        let inner_name = self.typ.name().to_camel();

        let define = def_builder_for_vector(&struct_name, &inner_name);
        let setter = impl_setters_for_vector(&struct_name, &inner_name);
        let default = impl_default_for_vector(&struct_name, &inner_name);

        let build = format!(
            r#"
func (s *{struct_name}Builder) Build() {struct_name} {{
    size := packNumber(Number(len(s.inner)))

    b := new(bytes.Buffer)

    b.Write(size)
    len := len(s.inner)
    for i := 0; i < len; i++ {{
        b.Write(s.inner[i].AsSlice())
    }}

    sb := {struct_name}{{inner: b.Bytes()}}

    return sb
}}
            "#,
            struct_name = struct_name
        );

        vec![define, build, setter, default].join("\n")
    }
}

impl GenBuilder for ast::DynVec {
    fn gen_builder(&self) -> String {
        let struct_name = self.name().to_camel();
        let inner_name = self.typ.name().to_camel();

        let define = def_builder_for_vector(&struct_name, &inner_name);
        let setter = impl_setters_for_vector(&struct_name, &inner_name);
        let default = impl_default_for_vector(&struct_name, &inner_name);
        let build = format!(
            r#"
func (s *{struct_name}Builder) Build() {struct_name} {{
    itemCount := len(s.inner)
    size := packNumber(Number(itemCount))

    b := new(bytes.Buffer)

    // Empty dyn vector, just return size's bytes
    if itemCount == 0 {{
        b.Write(size)
        return {struct_name}{{inner: b.Bytes()}}
    }}

    // Calculate first offset then loop for rest items offsets
    totalSize := HeaderSizeUint * uint32(itemCount+1)
    offsets := make([]uint32, 0, itemCount)
    offsets = append(offsets, totalSize)
    for i := 1; i < itemCount; i++ {{
        totalSize += uint32(len(s.inner[i-1].AsSlice()))
        offsets = append(offsets, offsets[i-1]+uint32(len(s.inner[i-1].AsSlice())))
    }}
    totalSize += uint32(len(s.inner[itemCount-1].AsSlice()))

    b.Write(packNumber(Number(totalSize)))

    for i := 0; i < itemCount; i++ {{
        b.Write(packNumber(Number(offsets[i])))
    }}

    for i := 0; i < itemCount; i++ {{
        b.Write(s.inner[i].AsSlice())
    }}

    return {struct_name}{{inner: b.Bytes()}}
}}
          "#,
            struct_name = struct_name
        );
        vec![define, build, setter, default].join("\n")
    }
}

impl GenBuilder for ast::Table {
    fn gen_builder(&self) -> String {
        let field_count = self.inner.len();
        let struct_name = self.name().to_camel();

        let define = def_builder_for_struct_or_table(&struct_name, &self.inner[..]);
        let setter = impl_setters_for_struct_or_table(&struct_name, &self.inner[..]);
        let default = impl_default_for_struct_or_table(&struct_name, &self.inner[..]);

        let build = if self.inner.is_empty() {
            format!(
                r#"
func (s *{struct_name}Builder) Build() {struct_name} {{
    s := new(bytes.Buffer)
    s.Write(packNumber(Number(HeaderSizeUint)))
}}
                "#,
                struct_name = struct_name
            )
        } else {
            let fields_offset = self
                .inner
                .iter()
                .map(|f| {
                    let field_name = &f.name;
                    format!("offsets = append(offsets, totalSize)\ntotalSize += uint32(len(s.{}.AsSlice()))", field_name)
                })
                .collect::<Vec<String>>()
                .join("\n");
            let fields_encode = self
                .inner
                .iter()
                .map(|f| {
                    let field_name = &f.name;
                    format!("b.Write(s.{}.AsSlice())", field_name)
                })
                .collect::<Vec<String>>()
                .join("\n");

            format!(
                r#"
func (s *{struct_name}Builder) Build() {struct_name} {{
    b := new(bytes.Buffer)

    totalSize := HeaderSizeUint * ({field_count} + 1)
    offsets := make([]uint32, 0, {field_count})

    {fields_offset}

    b.Write(packNumber(Number(totalSize)))

    for i := 0; i < len(offsets); i++ {{
        b.Write(packNumber(Number(offsets[i])))
    }}

    {fields_encode}
    return {struct_name}{{inner: b.Bytes()}}
}}
                "#,
                struct_name = struct_name,
                fields_offset = fields_offset,
                fields_encode = fields_encode,
                field_count = field_count
            )
        };
        vec![define, build, setter, default].join("\n")
    }
}

fn def_builder_for_struct_or_table(struct_name: &str, inner: &[ast::FieldDecl]) -> String {
    let fields = inner
        .iter()
        .map(|f| {
            let field_name = &f.name;
            let field_type = f.typ.name().to_camel();
            format!("{} {}", field_name, field_type)
        })
        .collect::<Vec<String>>()
        .join("\n");

    format!(
        r#"
type {struct_name}Builder struct {{
    {fields}
}}
        "#,
        struct_name = struct_name,
        fields = fields
    )
}

fn impl_default_for_struct_or_table(struct_name: &str, inner: &[ast::FieldDecl]) -> String {
    let each_field = inner
        .iter()
        .map(|f| {
            let field_name = &f.name;
            let field_type = f.typ.name().to_camel();
            format!(
                "{field_name}: {field_type}Default()",
                field_name = field_name,
                field_type = field_type
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#"
func New{struct_name}Builder() *{struct_name}Builder {{
	return &{struct_name}Builder{{ {each_field} }}
}}
    "#,
        struct_name = struct_name,
        each_field = each_field
    )
}

fn impl_setters_for_struct_or_table(struct_name: &str, inner: &[ast::FieldDecl]) -> String {
    inner
        .iter()
        .map(|f| {
            let func_name = f.name.to_camel();
            let field_name = &f.name;
            let field_type = f.typ.name().to_camel();
            format!(
                r#"
func (s *{struct_name}Builder) {func_name}(v {field_type}) *{struct_name}Builder {{
    s.{field_name} = v
    return s
}}
            "#,
                struct_name = struct_name,
                field_name = field_name,
                field_type = field_type,
                func_name = func_name
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(in super::super) fn impl_as_builder_for_struct_or_table(
    struct_name: &str,
    inner: &[ast::FieldDecl],
) -> String {
    let each_field = inner
        .iter()
        .map(|f| {
            let func_name = f.name.to_camel();
            format!(".{func_name}(*s.{func_name}())", func_name = func_name)
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        r#"
func (s *{struct_name}) AsBuilder() {struct_name}Builder {{
    ret := New{struct_name}Builder(){each_field}
    return *ret
}}
        "#,
        struct_name = struct_name,
        each_field = each_field
    )
}

fn def_builder_for_vector(struct_name: &str, inner_name: &str) -> String {
    format!(
        r#"
type {struct_name}Builder struct {{
    inner []{inner_type}
}}
    "#,
        struct_name = struct_name,
        inner_type = inner_name
    )
}

fn impl_setters_for_vector(struct_name: &str, inner_name: &str) -> String {
    format!(
        r#"
func (s *{struct_name}Builder) Set(v []{inner_name}) *{struct_name}Builder {{
    s.inner = v
    return s
}}
func (s *{struct_name}Builder) Push(v {inner_name}) *{struct_name}Builder {{
    s.inner = append(s.inner, v)
    return s
}}
func (s *{struct_name}Builder) Extend(iter []{inner_name}) *{struct_name}Builder {{
    for i:=0; i < len(iter); i++ {{
        s.inner = append(s.inner, iter[i])
    }}
    return s
}}
    "#,
        struct_name = struct_name,
        inner_name = inner_name
    )
}

fn impl_default_for_vector(struct_name: &str, inner_name: &str) -> String {
    format!(
        r#"
func New{struct_name}Builder() *{struct_name}Builder {{
	return &{struct_name}Builder{{ []{inner_name}{{}} }}
}}
        "#,
        struct_name = struct_name,
        inner_name = inner_name
    )
}

pub(in super::super) fn impl_as_builder_for_vector(struct_name: &str) -> String {
    format!(
        r#"
func (s *{struct_name}) AsBuilder() {struct_name}Builder {{
    size := s.ItemCount()
    t := New{struct_name}Builder()
    for i:=uint(0); i < size; i++ {{
        t.Push(*s.Get(i))
    }}
    return *t
}}
        "#,
        struct_name = struct_name
    )
}
