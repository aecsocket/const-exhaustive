//! Derive macros for [`const-exhaustive`].
//!
//! [`const-exhaustive`]: https://docs.rs/const-exhaustive

use {
    proc_macro2::{Span, TokenStream},
    quote::{quote, ToTokens},
    syn::{
        parse_macro_input, parse_quote, Data, DataEnum, DataStruct, DeriveInput, Error, Field,
        Fields, Ident, Result, WherePredicate,
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
            ),* $(,)?
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
        core::marker:::Copy,
        core::mem:::MaybeUninit,
        core::ops:::Add,
        core::ops:::Mul,
        const_exhaustive:::Exhaustive,
        const_exhaustive:::const_transmute,
        const_exhaustive::typenum:::U0,
        const_exhaustive::typenum:::U1,
        const_exhaustive::typenum:::Unsigned,
        const_exhaustive::generic_array:::GenericArray,
        const_exhaustive::generic_array:::ArrayLength,
    }
}

fn derive(input: &DeriveInput) -> Result<TokenStream> {
    let Shortcuts {
        Exhaustive,
        MaybeUninit,
        GenericArray,
        const_transmute,
        ..
    } = Shortcuts::default();

    let ExhaustiveImpl {
        num,
        values,
        predicate,
    } = match &input.data {
        Data::Struct(data) => make_for_struct(data),
        Data::Enum(data) => make_for_enum(data),
        Data::Union(_) => {
            return Err(Error::new_spanned(
                input,
                "exhaustive union is not supported",
            ));
        }
    };

    let name = &input.ident;

    let mut generics = input.generics.clone();
    let type_params = generics
        .type_params()
        .map(|p| p.ident.clone())
        .collect::<Vec<_>>();
    for param in type_params {
        generics.make_where_clause().predicates.push(parse_quote! {
            #param: #Exhaustive
        });
    }
    generics.make_where_clause().predicates.push(predicate);

    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        unsafe impl #impl_generics #Exhaustive for #name #type_generics #where_clause {
            type Num = #num;

            const ALL: #GenericArray<Self, Self::Num> = {
                let mut all: #GenericArray<#MaybeUninit<Self>, Self::Num> = unsafe {
                    #MaybeUninit::uninit().assume_init()
                };

                let mut i = 0;
                #values

                unsafe { #const_transmute(all) }
            };
        }
    })
}

struct ExhaustiveImpl {
    num: TokenStream,
    values: TokenStream,
    predicate: WherePredicate,
}

fn make_for_struct(data: &DataStruct) -> ExhaustiveImpl {
    make_for_fields(&data.fields, quote! { Self })
}

fn make_for_enum(data: &DataEnum) -> ExhaustiveImpl {
    struct VariantInfo {
        num: TokenStream,
        values: TokenStream,
    }

    let Shortcuts {
        U0,
        U1,
        Add,
        ArrayLength,
        ..
    } = Shortcuts::default();

    let variants = data
        .variants
        .iter()
        .map(|variant| {
            let ident = &variant.ident;
            let ExhaustiveImpl {
                num,
                values,
                predicate,
            } = make_for_fields(&variant.fields, quote! { Self::#ident });
            VariantInfo { num, values }
        })
        .collect::<Vec<_>>();

    let num = variants
        .iter()
        .fold(quote! { #U0 }, |acc, VariantInfo { num, .. }| {
            quote! { <#acc as #Add<#num>>::Output }
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

    ExhaustiveImpl {
        num,
        values,
        // TODO
        predicate: parse_quote! { #U1: #ArrayLength<ArrayType<Self>: Copy> },
    }
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
        Mul,
        ArrayLength,
        Copy,
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

    // this one is right fold
    let num = fields
        .iter()
        .rfold(quote! { #U1 }, |acc, FieldInfo { field, .. }| {
            let ty = &field.ty;
            quote! {
                <#acc as #Mul<<#ty as #Exhaustive>::Num>>::Output
            }
        });

    // and this one is left fold
    // has to be the opposite folding order to `num`
    let bound = fields.iter().fold(
        quote! { #ArrayLength<ArrayType<Self>: #Copy> },
        |acc, FieldInfo { field, .. }| {
            let ty = &field.ty;
            quote! {
                #Mul<<#ty as #Exhaustive>::Num, Output: #acc>
            }
        },
    );
    let predicate = parse_quote! { #U1: #bound };

    // rfold here so that the value order matches the tuple value order
    // e.g. we generate i_0 { i_1 { i_2 } }
    //       instead of i_2 { i_1 { i_0 } }
    let values = fields.iter().rfold(
        quote! {
            all.as_mut_slice()[i] = #MaybeUninit::new(#construct_ident #construct);
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

    ExhaustiveImpl {
        num,
        values,
        predicate,
    }
}
