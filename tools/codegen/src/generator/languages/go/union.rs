use crate::ast::verified::{self as ast, HasName};

use case::CaseExt;

pub(in super::super) trait GenUnion {
    fn gen_union(&self) -> (String, String);
}

impl GenUnion for ast::Union {
    fn gen_union(&self) -> (String, String) {
        let struct_name = self.name().to_camel();
        let union_name = format!("{}Union", struct_name);

        let define = format!(
            r#"
            type {union_name} struct {{
                itemID Number
                inner []byte
            }}
            func (s *{union_name}) AsSlice() []byte {{
                return s.inner
            }}
            func (s *{union_name}) ItemID() Number {{
                return s.itemID
            }}
        "#,
            union_name = union_name
        );

        let inner_len = self.inner.len();
        let (union_items, union_ids, part_impl) = {
            self.inner.iter().enumerate().fold(
                (
                    Vec::with_capacity(inner_len),
                    Vec::with_capacity(inner_len),
                    Vec::with_capacity(inner_len),
                ),
                |(mut union_items, mut union_ids, mut part_impl), (index, inner)| {
                    let item_name = inner.typ.name();
                    let item_id = index;
                    union_items.push(item_name);
                    union_ids.push(item_id);
                    let part = format!(
                        r#"
                        func {union_name}From{item_name}(v {item_name}) {union_name} {{
                            return {union_name}{{itemID: {item_id}, inner: v.AsSlice()}}
                        }}
                        "#,
                        union_name = union_name,
                        item_name = item_name,
                        item_id = item_id
                    );
                    part_impl.push(part);
                    (union_items, union_ids, part_impl)
                },
            )
        };

        let union_switch_impl = union_ids
            .iter()
            .zip(union_items.iter())
            .map(|(id, item)| {
                format!(
                    r#"
                	case {}:
                        return "{}"
                "#,
                    id, item
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        let to_union_switch_iml = union_ids
            .iter()
            .map(|id| {
                format!(
                    r#"
                	case {id}:
                        return &{union_name}{{itemID: {id}, inner: s.inner[HeaderSizeUint:]}}
                "#,
                    id = id,
                    union_name = union_name
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        let from_slice_switch_iml = union_ids
            .iter()
            .zip(union_items.iter())
            .map(|(id, item)| {
                format!(
                    r#"
                	case {id}:
                        _, err := {item}FromSlice(innerSlice, compatible)
                        if err != nil {{
                            return nil, err
                        }}
                "#,
                    id = id,
                    item = item
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        let union_switch = format!(
            r#"
            func (s *{union_name}) ItemName() string {{
                switch s.itemID {{
                {union_switch_impl}
                default:
                    panic("invalid data: OrUnion")
                }}
            }}
            "#,
            union_switch_impl = union_switch_impl,
            union_name = union_name
        );

        let to_union = format!(
            r#"
        func (s *{struct_name}) ToUnion() *{union_name} {{
            switch s.ItemID() {{
            {to_union_switch_iml}
            default:
                panic("invalid data: Or")
            }}
        }}
        "#,
            to_union_switch_iml = to_union_switch_iml,
            union_name = union_name,
            struct_name = struct_name
        );

        (
            vec![define, part_impl.join("\n"), union_switch, to_union].join("\n"),
            from_slice_switch_iml,
        )
    }
}
