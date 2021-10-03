extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro2::Literal;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned};

// TODO:
// - include repr info, maybe even enforce it?
// - can and should it work for unsized types as well?
// - find work around for type graph capacity

#[proc_macro_derive(TypeLayout)]
pub fn derive_type_layout(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as syn::DeriveInput);

    // Used in the quasi-quotation below as `#ty_name`.
    let ty_name = input.ident;

    let mut consts = Vec::new();

    let ty_generics = input.generics.split_for_impl().1;
    let layout = layout_of_type(&ty_name, &ty_generics, &input.data, &mut consts);

    let mut input_generics_a = input.generics.clone();

    input_generics_a
        .make_where_clause()
        .predicates
        .push(syn::parse_quote! {
            [u8; ::core::mem::size_of::<::core::mem::Discriminant<#ty_name #ty_generics>>()]:
        });

    for param in input_generics_a.type_params_mut() {
        param
            .bounds
            .push(syn::parse_quote!(::type_layout::TypeLayout));
    }

    let mut input_generics_b = input.generics.clone();

    input_generics_b
        .make_where_clause()
        .predicates
        .push(syn::parse_quote! {
            [u8; ::core::mem::size_of::<::core::mem::Discriminant<#ty_name #ty_generics>>()]:
        });

    /*for param in input_generics_b.type_params_mut() {
        param
            .bounds
            .push(syn::parse_quote!(~const ::type_layout::TypeGraph));
    }*/

    let mut inner_types = Vec::new();

    match &input.data {
        syn::Data::Struct(syn::DataStruct { fields, .. }) => {
            for field in fields {
                inner_types.push(&field.ty);
            }
        }
        syn::Data::Union(syn::DataUnion {
            fields: syn::FieldsNamed { named: fields, .. },
            ..
        }) => {
            for field in fields {
                inner_types.push(&field.ty);
            }
        }
        syn::Data::Enum(syn::DataEnum { variants, .. }) => {
            for variant in variants {
                for field in &variant.fields {
                    inner_types.push(&field.ty);
                }
            }
        }
    }

    let (impl_generics_a, ty_generics_a, where_clause_a) = input_generics_a.split_for_impl();
    let (impl_generics_b, ty_generics_b, where_clause_b) = input_generics_b.split_for_impl();

    let type_bounds = input_generics_b
        .type_params()
        .map(|param| &param.ident)
        .collect::<Vec<_>>();

    let where_clause_b = match where_clause_b {
        Some(where_clause_b) if !where_clause_b.predicates.is_empty() => {
            let joiner = if where_clause_b.predicates.trailing_punct() {
                quote!()
            } else {
                quote!(,)
            };

            quote! {
                #where_clause_b #joiner #(#type_bounds: ~const ::type_layout::TypeGraph),*
            }
        }
        _ => {
            quote! {
                where #(#type_bounds: ~const ::type_layout::TypeGraph),*
            }
        }
    };

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        unsafe impl #impl_generics_a ::type_layout::TypeLayout for #ty_name #ty_generics_a #where_clause_a {
            const TYPE_LAYOUT: ::type_layout::TypeLayoutInfo<'static> = {
                ::type_layout::TypeLayoutInfo {
                    name: ::core::any::type_name::<Self>(),
                    size: ::core::mem::size_of::<Self>(),
                    alignment: ::core::mem::align_of::<Self>(),
                    structure: #layout,
                }
            };
        }

        #(impl #impl_generics_a #ty_name #ty_generics_a #where_clause_a {
            #consts
        })*

        unsafe impl #impl_generics_b const ::type_layout::TypeGraph for #ty_name #ty_generics_b #where_clause_b {
            fn populate_graph(graph: &mut ::type_layout::TypeLayoutGraph<'static>) {
                if graph.insert(&Self::TYPE_LAYOUT) {
                    #(<#inner_types as ::type_layout::TypeGraph>::populate_graph(graph);)*
                }
            }
        }
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}

fn layout_of_type(
    ty_name: &syn::Ident,
    ty_generics: &syn::TypeGenerics,
    data: &syn::Data,
    consts: &mut Vec<proc_macro2::TokenStream>,
) -> proc_macro2::TokenStream {
    match data {
        syn::Data::Struct(data) => {
            let fields = quote_fields(
                ty_name,
                None,
                quote_field_values(ty_name, ty_generics, &data.fields),
                consts,
            );

            quote! {
                ::type_layout::TypeStructure::Struct { fields: #fields }
            }
        }
        syn::Data::Enum(r#enum) => {
            let variants = r#enum
                .variants
                .iter()
                .map(|variant| {
                    let variant_name = &variant.ident;
                    let variant_name_str = Literal::string(&variant_name.to_string());

                    let variant_constructor = match &variant.fields {
                        syn::Fields::Unit => quote! { #ty_name::#variant_name },
                        syn::Fields::Unnamed(fields) => {
                            let initialisers = fields.unnamed.iter().map(|_| {
                                quote! { unsafe { ::core::mem::MaybeUninit::uninit().assume_init() } }
                            }).collect::<Vec<_>>();

                            quote! { #ty_name::#variant_name(#(#initialisers),*) }
                        },
                        syn::Fields::Named(fields) => {
                            let initialisers = fields.named.iter().map(|field| {
                                let field_name = field.ident.as_ref().unwrap();

                                quote! { #field_name: unsafe { ::core::mem::MaybeUninit::uninit().assume_init() } }
                            }).collect::<Vec<_>>();

                            quote! { #ty_name::#variant_name { #(#initialisers),* } }
                        }
                    };

                    let variant_destructor = match &variant.fields {
                        syn::Fields::Unit => quote! { #ty_name::#variant_name },
                        syn::Fields::Unnamed(fields) => {
                            let destructors = fields.unnamed.iter().enumerate().map(|(field_index, _)| {
                                let field_name = quote::format_ident!("__self_{}", field_index);

                                quote! { #field_name }
                            }).collect::<Vec<_>>();

                            quote! { #ty_name::#variant_name(#(#destructors),*) }
                        },
                        syn::Fields::Named(fields) => {
                            let destructors = fields.named.iter().map(|field| {
                                let field_name = field.ident.as_ref().unwrap();

                                quote! { #field_name }
                            }).collect::<Vec<_>>();

                            quote! { #ty_name::#variant_name { #(#destructors),* } }
                        }
                    };

                    let fields = quote_fields(ty_name, Some(variant_name), match &variant.fields {
                        syn::Fields::Named(fields) => {
                            fields.named.iter().map(|field| {
                                let field_name = field.ident.as_ref().unwrap();
                                let field_name_str = Literal::string(&field_name.to_string());
                                let field_ty = &field.ty;

                                quote_spanned! { field.span() =>
                                    ::type_layout::Field {
                                        name: #field_name_str,
                                        offset: {
                                            let __variant_base: ::core::mem::MaybeUninit<#ty_name #ty_generics> = ::core::mem::MaybeUninit::new(#variant_constructor);

                                            #[allow(unused_variables, unreachable_patterns)]
                                            match unsafe { __variant_base.assume_init_ref() } {
                                                #variant_destructor => unsafe {
                                                    (#field_name as *const #field_ty as *const u8).offset_from(__variant_base.as_ptr() as *const u8) as usize
                                                },
                                                _ => unreachable!(),
                                            }
                                        },
                                        ty: ::core::any::type_name::<#field_ty>(),
                                    }
                                }
                            }).collect()
                        },
                        syn::Fields::Unnamed(fields) => {
                            fields.unnamed.iter().enumerate().map(|(field_index, field)| {
                                let field_name = quote::format_ident!("__self_{}", field_index);
                                let field_name_str = Literal::string(&field_index.to_string());
                                let field_ty = &field.ty;

                                quote_spanned! { field.span() =>
                                    ::type_layout::Field {
                                        name: #field_name_str,
                                        offset: {
                                            let __variant_base: ::core::mem::MaybeUninit<#ty_name #ty_generics> = ::core::mem::MaybeUninit::new(#variant_constructor);

                                            #[allow(unused_variables, unreachable_patterns)]
                                            match unsafe { __variant_base.assume_init_ref() } {
                                                #variant_destructor => unsafe {
                                                    (#field_name as *const #field_ty as *const u8).offset_from(__variant_base.as_ptr() as *const u8) as usize
                                                },
                                                _ => unreachable!(),
                                            }
                                        },
                                        ty: ::core::any::type_name::<#field_ty>(),
                                    }
                                }
                            }).collect()
                        },
                        syn::Fields::Unit => vec![],
                    }, consts);

                    let ident = syn::Ident::new(
                        &format!("__{}_{}_discriminant", ty_name, variant_name).to_uppercase(),
                        ty_name.span(),
                    );

                    consts.push(quote! {
                        const #ident: [u8; ::core::mem::size_of::<::core::mem::Discriminant<#ty_name #ty_generics>>()] = unsafe {
                            let variant: ::core::mem::MaybeUninit<#ty_name #ty_generics> = ::core::mem::MaybeUninit::new(#variant_constructor);

                            let system_endian_bytes: [u8; ::core::mem::size_of::<::core::mem::Discriminant<#ty_name #ty_generics>>()] = ::core::mem::transmute(::core::mem::discriminant(variant.assume_init_ref()));

                            let mut big_endian_bytes = [0_u8; ::core::mem::size_of::<::core::mem::Discriminant<#ty_name #ty_generics>>()];

                            let mut i = 0;

                            while i < system_endian_bytes.len() {
                                big_endian_bytes[i] = system_endian_bytes[if cfg!(target_endian = "big") {
                                    i
                                } else /* cfg!(target_endian = "little") */ {
                                    system_endian_bytes.len() - i - 1
                                }];

                                i += 1;
                            }

                            big_endian_bytes
                        };
                    });

                    quote! {
                        ::type_layout::Variant {
                            name: #variant_name_str,
                            discriminant: ::type_layout::Discriminant {
                                big_endian_bytes: &Self::#ident
                            },
                            fields: #fields,
                        }
                    }
                })
                .collect::<Vec<_>>();

            let variants_len = variants.len();

            let ident = syn::Ident::new(
                &format!("__{}_variants", ty_name).to_uppercase(),
                ty_name.span(),
            );

            consts.push(quote! {
                const #ident: &'static [::type_layout::Variant<'static>; #variants_len] = &[#(#variants),*];
            });

            quote! {
                ::type_layout::TypeStructure::Enum { variants: Self::#ident }
            }
        }
        syn::Data::Union(union) => {
            let values = union.fields.named.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_name_str = Literal::string(&field_name.to_string());
                let field_ty = &field.ty;

                quote_spanned! { field.span() =>
                    ::type_layout::Field {
                        name: #field_name_str,
                        offset: ::type_layout::memoffset::offset_of_union!(#ty_name #ty_generics, #field_name),
                        ty: ::core::any::type_name::<#field_ty>(),
                    }
                }
            }).collect();

            let fields = quote_fields(ty_name, None, values, consts);

            quote! {
                ::type_layout::TypeStructure::Union { fields: #fields }
            }
        }
    }
}

fn quote_field_values(
    ty_name: &syn::Ident,
    ty_generics: &syn::TypeGenerics,
    fields: &syn::Fields,
) -> Vec<proc_macro2::TokenStream> {
    match fields {
        syn::Fields::Named(fields) => {
            fields.named.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_name_str = Literal::string(&field_name.to_string());
                let field_ty = &field.ty;

                quote_spanned! { field.span() =>
                    ::type_layout::Field {
                        name: #field_name_str,
                        offset: ::type_layout::memoffset::offset_of!(#ty_name #ty_generics, #field_name),
                        ty: ::core::any::type_name::<#field_ty>(),
                    }
                }
            }).collect()
        },
        syn::Fields::Unnamed(fields) => {
            fields.unnamed.iter().enumerate().map(|(field_index, field)| {
                let field_name = syn::Index::from(field_index);
                let field_name_str = Literal::string(&field_index.to_string());
                let field_ty = &field.ty;

                quote_spanned! { field.span() =>
                    ::type_layout::Field {
                        name: #field_name_str,
                        offset: ::type_layout::memoffset::offset_of!(#ty_name #ty_generics, #field_name),
                        ty: ::core::any::type_name::<#field_ty>(),
                    }
                }
            }).collect()
        },
        syn::Fields::Unit => vec![],
    }
}

fn quote_fields(
    ty_name: &syn::Ident,
    qualifier: Option<&syn::Ident>,
    values: Vec<proc_macro2::TokenStream>,
    consts: &mut Vec<proc_macro2::TokenStream>,
) -> proc_macro2::TokenStream {
    let fields_len = values.len();

    let ident = syn::Ident::new(
        &(if let Some(qualifier) = qualifier {
            format!("__{}_{}_fields", ty_name, qualifier)
        } else {
            format!("__{}_fields", ty_name)
        })
        .to_uppercase(),
        ty_name.span(),
    );

    consts.push(quote! {
        const #ident: &'static [::type_layout::Field<'static>; #fields_len] = &[#(#values),*];
    });

    quote! { Self :: #ident }
}
