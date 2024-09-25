//! Derive macros for [`const-exhaustive`].
//!
//! [`const-exhaustive`]: https://docs.rs/const-exhaustive

use {
    proc_macro2::{Span, TokenStream},
    quote::quote,
    syn::{Data, DeriveInput, Field, Fields, FieldsNamed, Ident, Result, parse_macro_input},
};

#[proc_macro_derive(Exhaustive)]
pub fn exhaustive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn derive(input: &DeriveInput) -> Result<TokenStream> {
    // match &input.data {
    //     Data::Struct(data) => {
    //         match &data.fields {
    //             Fields::Named(fields) => {
    //                 fields.named
    //             }
    //         }
    //     }
    // }

    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => fields,
            _ => todo!(),
        },
        _ => todo!(),
    };
    let ExhaustiveImpl { num, all } = make_all(fields);

    Ok(quote! {
        unsafe impl #impl_generics ::const_exhaustive::Exhaustive for #name #type_generics #where_clause {
            type Num = #num;

            const ALL: ::const_exhaustive::generic_array::GenericArray<Self, Self::Num> = {
                #all
            };
        }
    })
}

struct ExhaustiveImpl {
    num: TokenStream,
    all: TokenStream,
}

fn make_all(fields: &FieldsNamed) -> ExhaustiveImpl {
    struct FieldInfo<'a> {
        field: &'a Field,
        ident: &'a Ident,
        index: Ident,
    }

    let fields = fields
        .named
        .iter()
        .map(|field| {
            let ident = field
                .ident
                .as_ref()
                .expect("named field must have an ident");
            let index = Ident::new(&format!("i_{ident}"), Span::call_site());
            FieldInfo {
                field,
                ident,
                index,
            }
        })
        .collect::<Vec<_>>();

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

    let construct = fields
        .iter()
        .map(
            |FieldInfo {
                 field,
                 ident,
                 index,
             }| {
                let ty = &field.ty;
                quote! {
                    #ident: <#ty as Exhaustive>::ALL.as_slice()[#index]
                }
            },
        )
        .collect::<Vec<_>>();

    let body = fields.iter().fold(
        quote! {
            let ptr = all.as_slice()[i].get();
            unsafe {
                *ptr = MaybeUninit::new(Self { #(#construct),* });
            };
            i += 1;
        },
        |acc,
         FieldInfo {
             field,
             ident,
             index,
         }| {
            let ty = &field.ty;
            quote! {
                let mut #index = 0usize;
                while #index < <#ty as Exhaustive>::Num::USIZE {
                    #acc
                    #index += 1;
                };
            }
        },
    );

    let all = quote! {
        use {
            ::const_exhaustive::{
                __util::const_transmute, generic_array::GenericArray, typenum::Unsigned, Exhaustive,
            },
            ::core::{cell::UnsafeCell, mem::MaybeUninit},
        };

        let all: GenericArray<UnsafeCell<MaybeUninit<Self>>, Self::Num> =
            unsafe { MaybeUninit::uninit().assume_init() };

        let mut i = 0;

        #body

        unsafe { const_transmute(all) }
    };

    ExhaustiveImpl { num, all }
}
