//! Derive macros for [`const-exhaustive`].
//!
//! [`const-exhaustive`]: https://docs.rs/const-exhaustive

use {
    proc_macro2::{Span, TokenStream},
    quote::quote,
    syn::{Data, DeriveInput, Fields, FieldsNamed, Ident, Result, parse_macro_input},
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
    let all = make_all(fields);

    Ok(quote! {
        unsafe impl #impl_generics ::const_exhaustive::Exhaustive for #name #type_generics #where_clause {
            type Num = ::const_exhaustive::typenum::UTerm;

            const ALL: ::const_exhaustive::generic_array::GenericArray<Self, Self::Num> = {
                #all
            };
        }
    })
}

fn make_all(fields: &FieldsNamed) -> TokenStream {
    let construct = fields
        .named
        .iter()
        .map(|field| {
            let ident = field
                .ident
                .as_ref()
                .expect("named field must have an ident");
            let index = Ident::new(&format!("i_{ident}"), Span::call_site());
            let ty = &field.ty;
            quote! {
                #ident: <#ty as Exhaustive>::ALL.as_slice()[#index]
            }
        })
        .collect::<Vec<_>>();

    let body = fields.named.iter().fold(
        quote! {
            let ptr = all.as_slice()[i].get();
            unsafe {
                *ptr = MaybeUninit::new(Self { #(#construct),* });
            };
            i += 1;
        },
        |acc, field| {
            let ident = field
                .ident
                .as_ref()
                .expect("named field must have an ident");
            let index = Ident::new(&format!("i_{ident}"), Span::call_site());
            let ty = &field.ty;
            quote! {
                let mut #index = 0usize;
                while (#index) < <#ty as Exhaustive>::Num::USIZE {
                    #acc
                    #index += 1;
                };
            }
        },
    );

    quote! {
        use {
            ::const_exhaustive::{
                __util::const_transmute, generic_array::GenericArray, typenum::Unsigned, Exhaustive,
            },
            ::core::{cell::UnsafeCell, mem::MaybeUninit},
        };

        let all = GenericArray<UnsafeCell<MaybeUninit<Self>>, Self::Num> =
            unsafe { MaybeUninit::uninit().assume_init() };

        let mut i = 0;

        #body

        unsafe { const_transmute(all) }
    }
}
