//! Derive macros for [`const-exhaustive`].
//!
//! [`const-exhaustive`]: https://docs.rs/const-exhaustive

use {
    core::iter::once,
    proc_macro2::{Span, TokenStream},
    quote::{quote, ToTokens},
    syn::{
        parse_macro_input, parse_quote, Data, DataEnum, DataStruct, DeriveInput, Error, Field,
        Fields, Ident, Result, Type, WherePredicate,
    },
};

/// Derives `const_exhaustive::Exhaustive` on this type.
///
/// This type must be [`Clone`] and [`Copy`], and all types contained within
/// it must also be `Exhaustive`.
///
/// The type may have type parameters (see *Limitations*).
///
/// Be warned that if a type is `Exhaustive`, then changing any of its fields
/// becomes a semver hazard.
///
/// # Limitations
///
/// This macro cannot be used on `union`s.
///
/// This macro cannot be used on generic `enum`s whose first variants do not
/// use a type parameter. This is a bug in the Rust compiler not propagating
/// type bounds properly. Here's a minimal reproduction of the issue:
///
/// ```ignore
/// use typenum::{U0, U1, U2};
/// use generic_array::ArrayLength;
/// use core::ops::Add;
///
/// trait Exhaustive {
///     type Num;
/// }
///
/// impl Exhaustive for bool {
///     type Num = U2;
/// }
///
/// struct X<T>(T);
/// impl<T> Exhaustive for X<T>
/// where
///     T: Exhaustive,
///     U0: Add<U1, Output: Add<<T as Exhaustive>::Num, Output: ArrayLength>>, // broken
///     // U0: Add<U1, Output: ArrayLength> // works
///     // U0: Add<U1, Output: Add<<T as Exhaustive>::Num>>, // works
///     // U0: Add<U1, Output: Add<<bool as Exhaustive>::Num, Output: ArrayLength>>, // works
/// {
///     type Num = U0;
/// }
/// ```
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
        core::marker:::Sized,
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

// general description of how the macro works:
// - works on a set of fields
//   - this could be the `{ .. }` in `SomeStruct { .. }`
//   - or the `{ .. }` in `SomeEnum::Variant { .. }`
// - make a `all` array, which will be the result of `Exhaustive::ALL`
// - make a `i`, which is the current index into `all` that we're writing
// - for each field:
//   - figure out `Num`
//     - make a `typenum` type which represents how many values that field has
//       - e.g. a `my_field: bool` has `<bool as Exhaustive>::Num` values
//       - done by multiplying the `Num` of each field type together
//       - e.g. in a `(bool, Option<bool>)` `Num` is `<bool::Num as Mul<
//         <Option<bool>>::Num >>::Output`
//     - write a where-clause predicate that expresses that the type we've made
//       above is bounded by `ArrayLength<ArrayType<Self>: Copy>`
//       - required when deriving on a type with type parameters
//       - this is structured as `U1: #bound`
//       - `#bound` starts off as `ArrayType<Self>: Copy`
//       - for each field, we wrap the existing `#bound` in `Mul<#field_ty::Num,
//         Output: #bound>`
//   - figure out `ALL`
//     - write all its values into `all`, and increase `i` accordingly
//
// - for structs, we take the resulting `Num` and `all`, and use those directly
//
// - for enums
//   - we take all the type bounds we've made from the field sets, and add those
//     to the `where` clause
//   - we also compute a final where predicate, in the form of `U0: #bound`
//     - starts off as `U0: ArrayLength<ArrayType<Self>: Copy>`
//     - for each field, we wrap the existing `#bound` in `Add<#field_ty::Num,
//       Output: #bound>`
//   - take all those where predicates, plus our final predicate, put them in
//     the impl block
//   - take the resulting `Num` and `all`, and put those into `Num` and `ALL`

fn derive(input: &DeriveInput) -> Result<TokenStream> {
    let Shortcuts {
        Exhaustive,
        MaybeUninit,
        GenericArray,
        Sized,
        Copy,
        const_transmute,
        ..
    } = Shortcuts::default();

    let ExhaustiveImpl {
        num,
        values,
        predicates,
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
    generics.make_where_clause().predicates.push(parse_quote! {
        // same bounds as `Exhaustive`
        Self: #Sized + #Copy
    });
    generics.make_where_clause().predicates.extend(predicates);

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
    predicates: Vec<WherePredicate>,
}

fn make_for_struct(data: &DataStruct) -> ExhaustiveImpl {
    make_for_fields(&data.fields, quote! { Self })
}

fn make_for_enum(data: &DataEnum) -> ExhaustiveImpl {
    let Shortcuts {
        U0,
        Add,
        ArrayLength,
        Copy,
        ..
    } = Shortcuts::default();

    let variants = data
        .variants
        .iter()
        .map(|variant| {
            let ident = &variant.ident;
            make_for_fields(&variant.fields, quote! { Self::#ident })
        })
        .collect::<Vec<_>>();

    // note the order of folds below:
    // - `Num` definition is left fold
    // - `Num` bound is right fold, opposite of `Num` definition
    // this is the opposite of what structs need, how interesting!
    // I wonder why?

    let num = variants
        .iter()
        .fold(quote! { #U0 }, |acc, ExhaustiveImpl { num, .. }| {
            quote! { <#acc as #Add<#num>>::Output }
        });

    let bound = variants.iter().rfold(
        quote! { #ArrayLength<ArrayType<Self>: #Copy> },
        |acc, ExhaustiveImpl { num, .. }| {
            quote! {
                #Add<#num, Output: #acc>
            }
        },
    );
    let predicate = parse_quote! { #U0: #bound };
    let predicates = variants
        .iter()
        .flat_map(|e| e.predicates.clone())
        .chain(once(predicate))
        .collect::<Vec<_>>();

    let values = variants
        .iter()
        .map(|ExhaustiveImpl { values, .. }| {
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
        predicates,
    }
}

fn make_for_fields(fields: &Fields, construct_ident: impl ToTokens) -> ExhaustiveImpl {
    struct FieldInfo<'a> {
        field: &'a Field,
        index: Ident,
        ty: &'a Type,
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
                    FieldInfo {
                        field,
                        index,
                        ty: &field.ty,
                    }
                })
                .collect::<Vec<_>>();
            let construct = fields
                .iter()
                .map(|FieldInfo { field, index, .. }| get_value(&field.ty, index));
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
                    FieldInfo {
                        field,
                        index,
                        ty: &field.ty,
                    }
                })
                .collect::<Vec<_>>();
            let construct = fields
                .iter()
                .map(|FieldInfo { field, index, .. }| {
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

    // see comment about order of folds above

    let num = fields
        .iter()
        .rfold(quote! { #U1 }, |acc, FieldInfo { field, .. }| {
            let ty = &field.ty;
            quote! {
                <#acc as #Mul<<#ty as #Exhaustive>::Num>>::Output
            }
        });

    let bound = fields.iter().fold(
        quote! { #ArrayLength<ArrayType<Self>: #Copy> },
        |acc, FieldInfo { field, .. }| {
            let ty = &field.ty;
            quote! {
                #Mul<<#ty as #Exhaustive>::Num, Output: #acc>
            }
        },
    );
    let extra_predicate = parse_quote! { #U1: #bound };

    let predicates = fields
        .iter()
        .map(|FieldInfo { ty, .. }| {
            parse_quote! {
                #ty: #Exhaustive<Num: #ArrayLength<ArrayType<Self>: #Copy>>
            }
        })
        .chain(once(extra_predicate))
        .collect::<Vec<_>>();

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
        predicates,
    }
}
