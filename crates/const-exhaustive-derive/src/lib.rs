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
///
/// # Limitations
///
/// This macro cannot be used on `union`s.
///
/// This macro cannot yet be used on types with type parameters. This is
/// technically possible, but requires the macro to add more explicit `where`
/// bounds. Pull requests welcome!
#[proc_macro_derive(Exhaustive)]
pub fn exhaustive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

macro_rules! shortcuts {
    {
        struct $shortcuts_name:ident {
            $(
                $($item_path:ident::)* : $item_name:ident
            ),*
        }
    } => {
        #[allow(non_snake_case, reason = "shortcut items")]
        struct $shortcuts_name {
            $(
                $item_name: TokenStream,
            )*
        }

        impl Default for $shortcuts_name {
            fn default() -> Self {
                Self {
                    $(
                        $item_name: quote! { ::$($item_path::)*$item_name },
                    )*
                }
            }
        }
    };
}

shortcuts! {
    struct Shortcuts {
        core::cell:::UnsafeCell,
        core::mem:::MaybeUninit,
        const_exhaustive:::Exhaustive,
        const_exhaustive:::const_transmute,
        const_exhaustive::typenum:::U0,
        const_exhaustive::typenum:::U1,
        const_exhaustive::typenum:::Unsigned,
        const_exhaustive::typenum::operator_aliases:::Sum,
        const_exhaustive::typenum::operator_aliases:::Prod,
        const_exhaustive::generic_array:::GenericArray
    }
}

fn derive(input: &DeriveInput) -> Result<TokenStream> {
    let Shortcuts { Exhaustive, .. } = Shortcuts::default();

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
        unsafe impl #impl_generics #Exhaustive for #name #type_generics #where_clause {
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

    let Shortcuts { U0, Sum, .. } = Shortcuts::default();

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

    let num = variants
        .iter()
        .fold(quote! { #U0 }, |acc, VariantInfo { num, .. }| {
            quote! { #Sum<#acc, #num> }
        });

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

    const fn require_ident(field: &Field) -> &Ident {
        field
            .ident
            .as_ref()
            .expect("named field must have an ident")
    }

    fn get_value(ty: impl ToTokens, index: impl ToTokens) -> TokenStream {
        let Shortcuts { Exhaustive, .. } = Shortcuts::default();

        quote! {
            <#ty as #Exhaustive>::ALL.as_slice()[#index]
        }
    }

    let Shortcuts {
        MaybeUninit,
        Exhaustive,
        U1,
        Unsigned,
        Prod,
        ..
    } = Shortcuts::default();

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

    let num = fields
        .iter()
        .fold(quote! { #U1 }, |acc, FieldInfo { field, .. }| {
            let ty = &field.ty;
            quote! {
                #Prod<#acc, <#ty as #Exhaustive>::Num>
            }
        });

    // rfold here so that the value order matches the tuple value order
    // e.g. we generate i_0 { i_1 { i_2 } }
    //       instead of i_2 { i_1 { i_0 } }
    let values = fields.iter().rfold(
        quote! {
            unsafe {
                *all.as_slice()[i].get() = #MaybeUninit::new(#construct_ident #construct);
            };
            i += 1;
        },
        |acc, FieldInfo { field, index, .. }| {
            let ty = &field.ty;
            quote! {
                let mut #index = 0usize;
                while #index < <<#ty as #Exhaustive>::Num as #Unsigned>::USIZE {
                    #acc
                    #index += 1;
                };
            }
        },
    );

    ExhaustiveImpl { num, values }
}

fn impl_body(num: impl ToTokens, values: impl ToTokens) -> TokenStream {
    let Shortcuts {
        UnsafeCell,
        MaybeUninit,
        GenericArray,
        const_transmute,
        ..
    } = Shortcuts::default();

    quote! {
        type Num = #num;

        const ALL: #GenericArray<Self, Self::Num> = {
            let all: #GenericArray<#UnsafeCell<#MaybeUninit<Self>>, Self::Num> = unsafe {
                #MaybeUninit::uninit().assume_init()
            };

            let mut i = 0;

            #values

            unsafe {
                #const_transmute(all)
            }
        };
    }
}
