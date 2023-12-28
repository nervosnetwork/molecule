use proc_macro2 as m4;
use quote::quote;

use super::super::utilities::{builder_name, entity_name, field_name, usize_lit};
use crate::ast::{self as ast, HasName};

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

impl ast::Option_ {
    pub(crate) fn gen_from(&self) -> m4::TokenStream {
        let entity = entity_name(self.name());
        let item_name = entity_name(self.item().typ().name());
        quote!(
            impl From<#item_name> for #entity {
                fn from(value: #item_name) -> Self {
                    Self::new_builder().set(Some(value)).build()
                }
            }
        )
    }
}

impl ImplBuilder for ast::Option_ {
    fn impl_builder_internal(&self) -> m4::TokenStream {
        quote!(
            fn expected_length(&self) -> usize {
                self.0
                    .as_ref()
                    .map(|ref inner| inner.as_slice().len())
                    .unwrap_or(0)
            }
            fn write<W: molecule::io::Write>(&self, writer: &mut W) -> molecule::io::Result<()> {
                self.0
                    .as_ref()
                    .map(|ref inner| writer.write_all(inner.as_slice()))
                    .unwrap_or(Ok(()))
            }
        )
    }
}

impl ast::Union {
    pub(crate) fn gen_from(&self) -> m4::TokenStream {
        let entity = entity_name(self.name());
        self.items()
            .iter()
            .map(|item| {
                let item_name = entity_name(item.typ().name());
                quote!(
                    impl From<#item_name> for #entity {
                        fn from(value: #item_name) -> Self {
                            Self::new_builder().set(value).build()
                        }
                    }
                )
            })
            .collect()
    }
}

impl ImplBuilder for ast::Union {
    fn impl_builder_internal(&self) -> m4::TokenStream {
        quote!(
            fn expected_length(&self) -> usize {
                molecule::NUMBER_SIZE + self.0.as_slice().len()
            }
            fn write<W: molecule::io::Write>(&self, writer: &mut W) -> molecule::io::Result<()> {
                writer.write_all(&molecule::pack_number(self.0.item_id()))?;
                writer.write_all(self.0.as_slice())
            }
        )
    }
}

impl ImplBuilder for ast::Array {
    fn impl_builder_internal(&self) -> m4::TokenStream {
        let write_inners = {
            let idx = (0..self.item_count()).map(usize_lit).collect::<Vec<_>>();
            quote!(
                #(
                    writer.write_all(self.0[#idx].as_slice())?;
                )*
            )
        };
        quote!(
            fn expected_length(&self) -> usize {
                Self::TOTAL_SIZE
            }
            fn write<W: molecule::io::Write>(&self, writer: &mut W) -> molecule::io::Result<()> {
                #write_inners
                Ok(())
            }
        )
    }
}

impl ast::Array {
    pub(crate) fn gen_from(&self) -> m4::TokenStream {
        let entity = entity_name(self.name());
        let item_name = entity_name(self.item().typ().name());
        let n = self.item_count();
        let nth = (0..n).map(|i| quote::format_ident!("nth{}", i));
        quote!(
            impl From<[#item_name; #n]> for #entity {
                fn from(value: [#item_name; #n]) -> Self {
                    Self::new_builder().set(value).build()
                }
            }

            impl ::core::convert::TryFrom<&[#item_name]> for #entity {
                type Error = ::core::array::TryFromSliceError;
                fn try_from(value: &[#item_name]) -> Result<Self, ::core::array::TryFromSliceError> {
                    // Use TryFrom<&[T]> for &[T; n].
                    Ok(Self::new_builder().set(<&[#item_name; #n]>::try_from(value)?.clone()).build())
                }
            }

            impl From<#entity> for [#item_name; #n] {
                fn from(value: #entity) -> Self {
                    [#(value.#nth(),)*]
                }
            }
        )
    }
}

impl ImplBuilder for ast::Struct {
    fn impl_builder_internal(&self) -> m4::TokenStream {
        let fields = self.fields().iter().map(|f| {
            let field_name = field_name(f.name());
            quote!(
                writer.write_all(self.#field_name.as_slice())?;
            )
        });
        quote!(
            fn expected_length(&self) -> usize {
                Self::TOTAL_SIZE
            }
            fn write<W: molecule::io::Write>(&self, writer: &mut W) -> molecule::io::Result<()> {
                #( #fields )*
                Ok(())
            }
        )
    }
}

impl ImplBuilder for ast::FixVec {
    fn impl_builder_internal(&self) -> m4::TokenStream {
        let write_inners = quote!(for inner in &self.0[..] {
            writer.write_all(inner.as_slice())?;
        });
        quote!(
            fn expected_length(&self) -> usize {
                molecule::NUMBER_SIZE + Self::ITEM_SIZE * self.0.len()
            }
            fn write<W: molecule::io::Write>(&self, writer: &mut W) -> molecule::io::Result<()> {
                writer.write_all(&molecule::pack_number(self.0.len() as molecule::Number))?;
                #write_inners
                Ok(())
            }
        )
    }
}

impl ImplBuilder for ast::DynVec {
    fn impl_builder_internal(&self) -> m4::TokenStream {
        quote!(
            fn expected_length(&self) -> usize {
                molecule::NUMBER_SIZE * (self.0.len() + 1)
                    + self
                        .0
                        .iter()
                        .map(|inner| inner.as_slice().len())
                        .sum::<usize>()
            }
            fn write<W: molecule::io::Write>(&self, writer: &mut W) -> molecule::io::Result<()> {
                let item_count = self.0.len();
                if item_count == 0 {
                    writer.write_all(&molecule::pack_number(
                        molecule::NUMBER_SIZE as molecule::Number,
                    ))?;
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
        )
    }
}

fn gen_from_iter(name: &str, item_name: &str) -> m4::TokenStream {
    let entity = entity_name(name);
    let item_name = entity_name(item_name);
    quote!(
        impl ::core::iter::FromIterator<#item_name> for #entity {
            fn from_iter<T: IntoIterator<Item = #item_name>>(iter: T) -> Self {
                Self::new_builder().extend(iter).build()
            }
        }
    )
}

impl ast::FixVec {
    pub(crate) fn gen_from_iter(&self) -> m4::TokenStream {
        gen_from_iter(self.name(), self.item().typ().name())
    }
}

impl ast::DynVec {
    pub(crate) fn gen_from_iter(&self) -> m4::TokenStream {
        gen_from_iter(self.name(), self.item().typ().name())
    }
}

impl ImplBuilder for ast::Table {
    fn impl_builder_internal(&self) -> m4::TokenStream {
        if self.fields().is_empty() {
            quote!(
                fn expected_length(&self) -> usize {
                    molecule::NUMBER_SIZE
                }
                fn write<W: molecule::io::Write>(
                    &self,
                    writer: &mut W,
                ) -> molecule::io::Result<()> {
                    writer.write_all(&molecule::pack_number(
                        molecule::NUMBER_SIZE as molecule::Number,
                    ))?;
                    Ok(())
                }
            )
        } else {
            let field = &self
                .fields()
                .iter()
                .map(|f| field_name(f.name()))
                .collect::<Vec<_>>();
            quote!(
                fn expected_length(&self) -> usize {
                    molecule::NUMBER_SIZE * (Self::FIELD_COUNT + 1)
                        #(+ self.#field.as_slice().len())*
                }
                fn write<W: molecule::io::Write>(&self, writer: &mut W) -> molecule::io::Result<()> {
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
            )
        }
    }
}
