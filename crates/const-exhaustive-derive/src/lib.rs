//! Derive macros for [`const-exhaustive`].
//!
//! [`const-exhaustive`]: https://docs.rs/const-exhaustive

use {
    proc_macro2::{Span, TokenStream},
    quote::{ToTokens, quote},
    syn::{
        Data, DataEnum, DataStruct, DeriveInput, Error, Field, Fields, Ident, Result,
        parse_macro_input,
    },
};

/// Derives `const_exhaustive::Exhaustive` on this type.
///
/// This type must be [`Clone`] and [`Copy`], and all types contained within
/// it must also be `Exhaustive`.
#[proc_macro_derive(Exhaustive)]
pub fn exhaustive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn derive(input: &DeriveInput) -> Result<TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let ExhaustiveImpl { num, values } = match &input.data {
        Data::Struct(data) => make_for_struct(data),
        Data::Enum(data) => make_for_enum(data),
        Data::Union(_) => {
            return Err(Error::new_spanned(
                input,
                "exhaustive union is not supported",
            ));
        }
    };

    let body = impl_body(num, values);
    Ok(quote! {
        unsafe impl #impl_generics ::const_exhaustive::Exhaustive for #name #type_generics #where_clause {
            #body
        }
    })
}

struct ExhaustiveImpl {
    num: TokenStream,
    values: TokenStream,
}

fn make_for_struct(data: &DataStruct) -> ExhaustiveImpl {
    make_for_fields(&data.fields, quote! { Self })
}

fn make_for_enum(data: &DataEnum) -> ExhaustiveImpl {
    struct VariantInfo {
        num: TokenStream,
        values: TokenStream,
    }

    let variants = data
        .variants
        .iter()
        .map(|variant| {
            let ident = &variant.ident;
            let ExhaustiveImpl { num, values } =
                make_for_fields(&variant.fields, quote! { Self::#ident });
            VariantInfo { num, values }
        })
        .collect::<Vec<_>>();

    let num = variants.iter().fold(
        quote! { ::const_exhaustive::typenum::U0 },
        |acc, VariantInfo { num, .. }| {
            quote! {
                ::const_exhaustive::typenum::operator_aliases::Sum<
                    #acc,
                    #num,
                >
            }
        },
    );

    let values = variants
        .iter()
        .map(|VariantInfo { values, .. }| {
            quote! {
                {
                    #values
                }
            }
        })
        .collect::<Vec<_>>();
    let values = quote! {
        #(#values)*
    };

    ExhaustiveImpl { num, values }
}

fn make_for_fields(fields: &Fields, construct_ident: impl ToTokens) -> ExhaustiveImpl {
    struct FieldInfo<'a> {
        field: &'a Field,
        index: Ident,
    }

    fn require_ident(field: &Field) -> &Ident {
        field
            .ident
            .as_ref()
            .expect("named field must have an ident")
    }

    fn get_value(ty: impl ToTokens, index: impl ToTokens) -> TokenStream {
        quote! {
            <#ty as ::const_exhaustive::Exhaustive>::ALL.as_slice()[#index]
        }
    }

    let (fields, construct) = match fields {
        Fields::Unit => (Vec::<FieldInfo>::new(), quote! {}),
        Fields::Unnamed(fields) => {
            let fields = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    let index = Ident::new(&format!("i_{index}"), Span::call_site());
                    FieldInfo { field, index }
                })
                .collect::<Vec<_>>();
            let construct = fields
                .iter()
                .map(|FieldInfo { field, index }| get_value(&field.ty, index));
            let construct = quote! {
                (
                    #(#construct),*
                )
            };
            (fields, construct)
        }
        Fields::Named(fields) => {
            let fields = fields
                .named
                .iter()
                .map(|field| {
                    let ident = require_ident(field);
                    let index = Ident::new(&format!("i_{ident}"), Span::call_site());
                    FieldInfo { field, index }
                })
                .collect::<Vec<_>>();
            let construct = fields
                .iter()
                .map(|FieldInfo { field, index }| {
                    let ident = require_ident(field);
                    let get_value = get_value(&field.ty, index);
                    quote! { #ident: #get_value }
                })
                .collect::<Vec<_>>();
            let construct = quote! {
                {
                    #(#construct),*
                }
            };
            (fields, construct)
        }
    };

    let num = fields.iter().fold(
        quote! { ::const_exhaustive::typenum::U1 },
        |acc, FieldInfo { field, .. }| {
            let ty = &field.ty;
            quote! {
                ::const_exhaustive::typenum::operator_aliases::Prod<
                    #acc,
                    <#ty as ::const_exhaustive::Exhaustive>::Num,
                >
            }
        },
    );

    // rfold here so that the value order matches the tuple value order
    // e.g. we generate i_0 { i_1 { i_2 } }
    //       instead of i_2 { i_1 { i_0 } }
    let values = fields.iter().rfold(
        quote! {
            let ptr = all.as_slice()[i].get();
            unsafe {
                *ptr = ::core::mem::MaybeUninit::new(#construct_ident #construct);
            };
            i += 1;
        },
        |acc, FieldInfo { field, index, .. }| {
            let ty = &field.ty;
            quote! {
                let mut #index = 0usize;
                while #index < <<#ty as ::const_exhaustive::Exhaustive>::Num as ::const_exhaustive::typenum::Unsigned>::USIZE {
                    #acc
                    #index += 1;
                };
            }
        },
    );

    ExhaustiveImpl { num, values }
}

fn impl_body(num: impl ToTokens, values: impl ToTokens) -> TokenStream {
    quote! {
        type Num = #num;

        const ALL: ::const_exhaustive::generic_array::GenericArray<Self, Self::Num> = {
            let all: ::const_exhaustive::generic_array::GenericArray<
                ::core::cell::UnsafeCell<
                    ::core::mem::MaybeUninit<Self>
                >, Self::Num
            > = unsafe {
                ::core::mem::MaybeUninit::uninit().assume_init()
            };

            let mut i = 0;

            #values

            unsafe {
                ::const_exhaustive::const_transmute(all)
            }
        };
    }
}
