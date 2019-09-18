use molecule::{pack_number, Number, NUMBER_SIZE};

pub(crate) trait DefaultContent {
    fn default_content(&self) -> Vec<u8>;
}

impl DefaultContent for super::Atom {
    fn default_content(&self) -> Vec<u8> {
        vec![0]
    }
}

impl DefaultContent for super::Option_ {
    fn default_content(&self) -> Vec<u8> {
        Vec::new()
    }
}

impl DefaultContent for super::Union {
    fn default_content(&self) -> Vec<u8> {
        let item_id = 0;
        let inner_content = self.inner[item_id].typ.default_content();
        let total_size = NUMBER_SIZE + inner_content.len();
        let mut content = Vec::with_capacity(total_size);
        content.extend_from_slice(&pack_number(item_id as Number));
        content.extend_from_slice(&inner_content);
        content
    }
}

impl DefaultContent for super::Array {
    fn default_content(&self) -> Vec<u8> {
        vec![0; self.total_size()]
    }
}

impl DefaultContent for super::Struct {
    fn default_content(&self) -> Vec<u8> {
        vec![0; self.total_size()]
    }
}

impl DefaultContent for super::FixVec {
    fn default_content(&self) -> Vec<u8> {
        let item_count = 0;
        let mut content = Vec::with_capacity(NUMBER_SIZE);
        content.extend_from_slice(&pack_number(item_count as Number));
        content
    }
}

impl DefaultContent for super::DynVec {
    fn default_content(&self) -> Vec<u8> {
        let total_size = NUMBER_SIZE;
        let mut content = Vec::with_capacity(NUMBER_SIZE);
        content.extend_from_slice(&pack_number(total_size as Number));
        content
    }
}

impl DefaultContent for super::Table {
    fn default_content(&self) -> Vec<u8> {
        let field_count = self.inner.len();
        let (total_size, content) = if field_count == 0 {
            let total_size = NUMBER_SIZE;
            let mut content = Vec::with_capacity(total_size);
            content.extend_from_slice(&pack_number(total_size as Number));
            (total_size, content)
        } else {
            let (total_size, offsets, field_data) = self.inner.iter().fold(
                (
                    NUMBER_SIZE * (field_count + 1),
                    Vec::with_capacity(field_count),
                    Vec::with_capacity(field_count),
                ),
                |(mut current_offset, mut offsets, mut field_data), field| {
                    offsets.push(current_offset);
                    let data = field.typ.default_content();
                    current_offset += data.len();
                    field_data.push(data);
                    (current_offset, offsets, field_data)
                },
            );
            let mut content = Vec::with_capacity(total_size);
            content.extend_from_slice(&pack_number(total_size as Number));
            for offset in offsets.into_iter() {
                content.extend_from_slice(&pack_number(offset as Number));
            }
            for data in field_data.into_iter() {
                content.extend_from_slice(&data);
            }
            (total_size, content)
        };
        assert_eq!(content.len(), total_size);
        content
    }
}

impl DefaultContent for super::TopDecl {
    fn default_content(&self) -> Vec<u8> {
        match self {
            super::TopDecl::Atom(inner) => inner.default_content(),
            super::TopDecl::Option_(inner) => inner.default_content(),
            super::TopDecl::Union(inner) => inner.default_content(),
            super::TopDecl::Array(inner) => inner.default_content(),
            super::TopDecl::Struct(inner) => inner.default_content(),
            super::TopDecl::FixVec(inner) => inner.default_content(),
            super::TopDecl::DynVec(inner) => inner.default_content(),
            super::TopDecl::Table(inner) => inner.default_content(),
        }
    }
}
