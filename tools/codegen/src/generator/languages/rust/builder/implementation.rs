use proc_macro2 as m4;
use quote::quote;

use super::super::utilities::{builder_name, entity_name, field_name, usize_lit};
use crate::ast::verified::{self as ast, HasName};

pub(in super::super) trait ImplBuilder: HasName {
    fn impl_builder_internal(&self) -> m4::TokenStream;

    fn impl_builder(&self) -> m4::TokenStream {
        let builder = builder_name(self.name());
        let builder_string = builder.to_string();
        let entity = entity_name(self.name());
        let internal = self.impl_builder_internal();
        quote!(
            impl molecule::prelude::Builder for #builder {
                type Entity = #entity;
                const NAME: &'static str = #builder_string;
                #internal
                fn build(&self) -> Self::Entity {
                    let mut inner = Vec::with_capacity(self.expected_length());
                    self.write(&mut inner)
                        .unwrap_or_else(|_| panic!("{} build should be ok", Self::NAME));
                    #entity::new_unchecked(inner.into())
                }
            }
        )
    }
}

impl ImplBuilder for ast::Option_ {
    fn impl_builder_internal(&self) -> m4::TokenStream {
        quote!(
            fn expected_length(&self) -> usize {
                self.0.as_ref().map(|ref inner| inner.as_slice().len()).unwrap_or(0)
            }
            #[cfg(feature = "std")]
            fn write<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                self.0.as_ref().map(|ref inner| writer.write_all(inner.as_slice())).unwrap_or(Ok(()))
            }
            fn write_buf<B: molecule::bytes::BufMut>(&self, buf: &mut B) {
                self.0.as_ref().map(|ref inner| buf.put_slice(inner.as_slice()));
            }
        )
    }
}

impl ImplBuilder for ast::Union {
    fn impl_builder_internal(&self) -> m4::TokenStream {
        quote!(
            fn expected_length(&self) -> usize {
                molecule::NUMBER_SIZE + self.0.as_slice().len()
            }
            #[cfg(feature = "std")]
            fn write<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                writer.write_all(&molecule::pack_number(self.0.item_id()))?;
                writer.write_all(self.0.as_slice())
            }
            fn write_buf<B: molecule::bytes::BufMut>(&self, buf: &mut B) {
                buf.put_slice(&molecule::pack_number(self.0.item_id()));
                buf.put_slice(self.0.as_slice());
            }
        )
    }
}

impl ImplBuilder for ast::Array {
    fn impl_builder_internal(&self) -> m4::TokenStream {
        let write_inners = {
            let idx = (0..self.item_count).map(usize_lit).collect::<Vec<_>>();
            quote!(
                #(
                    writer.write_all(self.0[#idx].as_slice())?;
                )*
            )
        };
        let buf_inners = {
            let idx = (0..self.item_count).map(usize_lit).collect::<Vec<_>>();
            quote!(
                #(
                    buf.put_slice(self.0[#idx].as_slice());
                )*
            )
        };
        quote!(
            fn expected_length(&self) -> usize {
                Self::TOTAL_SIZE
            }
            #[cfg(feature = "std")]
            fn write<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                #write_inners
                Ok(())
            }
            fn write_buf<B: molecule::bytes::BufMut>(&self, buf: &mut B) {
                #( #buf_inners )*
            }
        )
    }
}

impl ImplBuilder for ast::Struct {
    fn impl_builder_internal(&self) -> m4::TokenStream {
        let fields = self.inner.iter().map(|f| {
            let field_name = field_name(&f.name);
            quote!(
                writer.write_all(self.#field_name.as_slice())?;
            )
        });
        let buf_fields = self.inner.iter().map(|f| {
            let field_name = field_name(&f.name);
            quote!(
                buf.put_slice(self.#field_name.as_slice());
            )
        });
        quote!(
            fn expected_length(&self) -> usize {
                Self::TOTAL_SIZE
            }
            #[cfg(feature = "std")]
            fn write<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                #( #fields )*
                Ok(())
            }
            fn write_buf<B: molecule::bytes::BufMut>(&self, buf: &mut B) {
                #( #buf_fields )*
            }
        )
    }
}

impl ImplBuilder for ast::FixVec {
    fn impl_builder_internal(&self) -> m4::TokenStream {
        let write_inners = quote!(for inner in &self.0[..] {
            writer.write_all(inner.as_slice())?;
        });
        let buf_inners = quote!(for inner in &self.0[..] {
            buf.put_slice(inner.as_slice());
        });
        quote!(
            fn expected_length(&self) -> usize {
                molecule::NUMBER_SIZE + Self::ITEM_SIZE * self.0.len()
            }
            #[cfg(feature = "std")]
            fn write<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                writer.write_all(&molecule::pack_number(self.0.len() as molecule::Number))?;
                #write_inners
                Ok(())
            }
            fn write_buf<B: molecule::bytes::BufMut>(&self, buf: &mut B) {
                buf.put_slice(&molecule::pack_number(self.0.len() as molecule::Number));
                #buf_inners
            }
        )
    }
}

impl ImplBuilder for ast::DynVec {
    fn impl_builder_internal(&self) -> m4::TokenStream {
        quote!(
            fn expected_length(&self) -> usize {
                molecule::NUMBER_SIZE * (self.0.len() + 1)
                    + self.0.iter().map(|inner| inner.as_slice().len()).sum::<usize>()
            }
            #[cfg(feature = "std")]
            fn write<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                let item_count = self.0.len();
                if item_count == 0 {
                    writer.write_all(&molecule::pack_number(molecule::NUMBER_SIZE as molecule::Number))?;
                } else {
                    let (total_size, offsets) = self.0.iter().fold(
                        (
                            molecule::NUMBER_SIZE * (item_count + 1),
                            Vec::with_capacity(item_count),
                        ),
                        |(start, mut offsets), inner| {
                            offsets.push(start);
                            (start + inner.as_slice().len(), offsets)
                        },
                    );
                    writer.write_all(&molecule::pack_number(total_size as molecule::Number))?;
                    for offset in offsets.into_iter() {
                        writer.write_all(&molecule::pack_number(offset as molecule::Number))?;
                    }
                    for inner in self.0.iter() {
                        writer.write_all(inner.as_slice())?;
                    }
                }
                Ok(())
            }
            fn write_buf<B: molecule::bytes::BufMut>(&self, buf: &mut B) {
                let item_count = self.0.len();
                if item_count == 0 {
                    buf.put_slice(&molecule::pack_number(molecule::NUMBER_SIZE as molecule::Number));
                } else {
                    let (total_size, offsets) = self.0.iter().fold(
                        (
                            molecule::NUMBER_SIZE * (item_count + 1),
                            Vec::with_capacity(item_count),
                        ),
                        |(start, mut offsets), inner| {
                            offsets.push(start);
                            (start + inner.as_slice().len(), offsets)
                        },
                    );
                    buf.put_slice(&molecule::pack_number(total_size as molecule::Number));
                    for offset in offsets.into_iter() {
                        buf.put_slice(&molecule::pack_number(offset as molecule::Number));
                    }
                    for inner in self.0.iter() {
                        buf.put_slice(inner.as_slice());
                    }
                }
            }
        )
    }
}

impl ImplBuilder for ast::Table {
    fn impl_builder_internal(&self) -> m4::TokenStream {
        if self.inner.is_empty() {
            quote!(
                fn expected_length(&self) -> usize {
                    molecule::NUMBER_SIZE
                }
                #[cfg(feature = "std")]
                fn write<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                    writer.write_all(&molecule::pack_number(molecule::NUMBER_SIZE as molecule::Number))?;
                    Ok(())
                }
                fn write_buf<B: molecule::bytes::BufMut>(&self, buf: &mut B) {
                    buf.put_slice(&molecule::pack_number(molecule::NUMBER_SIZE as molecule::Number));
                }
            )
        } else {
            let field = &self
                .inner
                .iter()
                .map(|f| field_name(&f.name))
                .collect::<Vec<_>>();
            quote!(
                fn expected_length(&self) -> usize {
                    molecule::NUMBER_SIZE * (Self::FIELD_COUNT + 1)
                        #(+ self.#field.as_slice().len())*
                }
                #[cfg(feature = "std")]
                fn write<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                    let mut total_size = molecule::NUMBER_SIZE * (Self::FIELD_COUNT + 1);
                    let mut offsets = Vec::with_capacity(Self::FIELD_COUNT);
                    #(
                        offsets.push(total_size);
                        total_size += self.#field.as_slice().len();
                    )*
                    writer.write_all(&molecule::pack_number(total_size as molecule::Number))?;
                    for offset in offsets.into_iter() {
                        writer.write_all(&molecule::pack_number(offset as molecule::Number))?;
                    }
                    #(
                        writer.write_all(self.#field.as_slice())?;
                    )*
                    Ok(())
                }
                fn write_buf<B: molecule::bytes::BufMut>(&self, buf: &mut B) {
                    let mut total_size = molecule::NUMBER_SIZE * (Self::FIELD_COUNT + 1);
                    let mut offsets = Vec::with_capacity(Self::FIELD_COUNT);
                    #(
                        offsets.push(total_size);
                        total_size += self.#field.as_slice().len();
                    )*
                    buf.put_slice(&molecule::pack_number(total_size as molecule::Number));
                    for offset in offsets.into_iter() {
                        buf.put_slice(&molecule::pack_number(offset as molecule::Number));
                    }
                    #(
                        buf.put_slice(self.#field.as_slice());
                    )*
                }
            )
        }
    }
}
