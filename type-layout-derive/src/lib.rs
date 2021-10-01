extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro2::Literal;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields, Index};

#[proc_macro_derive(TypeLayout)]
pub fn derive_type_layout(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Used in the quasi-quotation below as `#ty_name`.
    let ty_name = input.ident;

    let mut consts = Vec::new();

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let layout = layout_of_type(&ty_name, &ty_generics, &input.data, &mut consts);

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        impl #impl_generics ::type_layout::TypeLayout for #ty_name #ty_generics #where_clause {
            const TYPE_LAYOUT: ::type_layout::TypeLayoutInfo<'static> = {
                ::type_layout::TypeLayoutInfo {
                    name: ::core::any::type_name::<Self>(),
                    size: ::core::mem::size_of::<Self>(),
                    alignment: ::core::mem::align_of::<Self>(),
                    structure: #layout,
                }
            };
        }

        #(impl #impl_generics #ty_name #ty_generics #where_clause {
            #consts
        })*
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}

fn layout_of_type(
    ty_name: &syn::Ident,
    ty_generics: &syn::TypeGenerics,
    data: &Data,
    consts: &mut Vec<proc_macro2::TokenStream>,
) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => {
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
        Data::Enum(r#enum) => {
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
                                let field_name = quote::format_ident!("f_{}", field_index);

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
                        Fields::Named(fields) => {
                            fields.named.iter().map(|field| {
                                let field_name = field.ident.as_ref().unwrap();
                                let field_name_str = Literal::string(&field_name.to_string());
                                let field_ty = &field.ty;

                                quote_spanned! { field.span() =>
                                    ::type_layout::Field {
                                        name: #field_name_str,
                                        ty: ::core::any::type_name::<#field_ty>(),
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
                                        size: ::core::mem::size_of::<#field_ty>(),
                                        alignment: ::core::mem::align_of::<#field_ty>(),
                                    }
                                }
                            }).collect()
                        },
                        Fields::Unnamed(fields) => {
                            fields.unnamed.iter().enumerate().map(|(field_index, field)| {
                                let field_name = quote::format_ident!("f_{}", field_index);
                                let field_name_str = Literal::string(&field_index.to_string());
                                let field_ty = &field.ty;

                                quote_spanned! { field.span() =>
                                    ::type_layout::Field {
                                        name: #field_name_str,
                                        ty: ::core::any::type_name::<#field_ty>(),
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
                                        size: ::core::mem::size_of::<#field_ty>(),
                                        alignment: ::core::mem::align_of::<#field_ty>(),
                                    }
                                }
                            }).collect()
                        },
                        Fields::Unit => vec![],
                    }, consts);

                    quote! {
                        ::type_layout::Variant {
                            name: #variant_name_str,
                            discriminant: {
                                let __variant_base: ::core::mem::MaybeUninit<#ty_name #ty_generics> = ::core::mem::MaybeUninit::new(#variant_constructor);

                                let discriminant = ::core::mem::discriminant(unsafe { __variant_base.assume_init_ref() });

                                match ::core::mem::size_of::<::core::mem::Discriminant<#ty_name #ty_generics>>() {
                                    0 => 0_u128,
                                    1 => unsafe { ::core::mem::transmute_copy::<_, u8>(&discriminant) as u128 },
                                    2 => unsafe { ::core::mem::transmute_copy::<_, u16>(&discriminant) as u128 },
                                    4 => unsafe { ::core::mem::transmute_copy::<_, u32>(&discriminant) as u128 },
                                    8 => unsafe { ::core::mem::transmute_copy::<_, u64>(&discriminant) as u128 },
                                    16 => unsafe { ::core::mem::transmute_copy::<_, u128>(&discriminant) as u128 },
                                    _ => unreachable!(),
                                }
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
        Data::Union(union) => {
            let values = union.fields.named.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_name_str = Literal::string(&field_name.to_string());
                let field_ty = &field.ty;

                quote_spanned! { field.span() =>
                    ::type_layout::Field {
                        name: #field_name_str,
                        ty: ::core::any::type_name::<#field_ty>(),
                        offset: ::type_layout::memoffset::offset_of_union!(#ty_name #ty_generics, #field_name),
                        size: ::core::mem::size_of::<#field_ty>(),
                        alignment: ::core::mem::align_of::<#field_ty>(),
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
    fields: &Fields,
) -> Vec<proc_macro2::TokenStream> {
    match fields {
        Fields::Named(fields) => {
            fields.named.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_name_str = Literal::string(&field_name.to_string());
                let field_ty = &field.ty;

                quote_spanned! { field.span() =>
                    ::type_layout::Field {
                        name: #field_name_str,
                        ty: ::core::any::type_name::<#field_ty>(),
                        offset: ::type_layout::memoffset::offset_of!(#ty_name #ty_generics, #field_name),
                        size: ::core::mem::size_of::<#field_ty>(),
                        alignment: ::core::mem::align_of::<#field_ty>(),
                    }
                }
            }).collect()
        },
        Fields::Unnamed(fields) => {
            fields.unnamed.iter().enumerate().map(|(field_index, field)| {
                let field_name = Index::from(field_index);
                let field_name_str = Literal::string(&field_index.to_string());
                let field_ty = &field.ty;

                quote_spanned! { field.span() =>
                    ::type_layout::Field {
                        name: #field_name_str,
                        ty: ::core::any::type_name::<#field_ty>(),
                        offset: ::type_layout::memoffset::offset_of!(#ty_name #ty_generics, #field_name),
                        size: ::core::mem::size_of::<#field_ty>(),
                        alignment: ::core::mem::align_of::<#field_ty>(),
                    }
                }
            }).collect()
        },
        Fields::Unit => vec![],
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
