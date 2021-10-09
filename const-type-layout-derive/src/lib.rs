#![deny(clippy::pedantic)]
#![feature(iter_intersperse)]

extern crate proc_macro;

#[macro_use]
extern crate proc_macro_error;

use proc_macro::TokenStream;

use proc_macro2::Literal;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned};

// TODO:
// - wait for `const_heap` feature to be implemented for a workaround the graph
//   size limitation: https://github.com/rust-lang/rust/issues/79597

#[proc_macro_error]
#[proc_macro_derive(TypeLayout, attributes(layout))]
pub fn derive_type_layout(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as syn::DeriveInput);

    // Used in the quasi-quotation below as `#ty_name`.
    let ty_name = input.ident;
    let ty_generics = input.generics.split_for_impl().1;

    let Attributes {
        reprs,
        add_bounds,
        sub_bounds,
    } = parse_attributes(&ty_name, &ty_generics, &input.attrs);

    let mut consts = Vec::new();
    let layout = layout_of_type(&ty_name, &ty_generics, &input.data, &reprs, &mut consts);

    let inner_types = extract_inner_types(&input.data);

    let Generics {
        type_layout_input_generics,
        type_graph_input_generics,
    } = generate_generics(
        &ty_name,
        &ty_generics,
        &input.generics,
        matches!(input.data, syn::Data::Enum(_)),
        &inner_types,
        add_bounds,
        &sub_bounds,
    );
    let (type_layout_impl_generics, type_layout_ty_generics, type_layout_where_clause) =
        type_layout_input_generics.split_for_impl();
    let (type_graph_impl_generics, type_graph_ty_generics, type_graph_where_clause) =
        type_graph_input_generics.split_for_impl();

    quote! {
        unsafe impl #type_layout_impl_generics ::const_type_layout::TypeLayout for
            #ty_name #type_layout_ty_generics #type_layout_where_clause
        {
            const TYPE_LAYOUT: ::const_type_layout::TypeLayoutInfo<'static> = {
                ::const_type_layout::TypeLayoutInfo {
                    name: ::core::any::type_name::<Self>(),
                    size: ::core::mem::size_of::<Self>(),
                    alignment: ::core::mem::align_of::<Self>(),
                    structure: #layout,
                }
            };
        }

        #(impl #type_layout_impl_generics #ty_name #type_layout_ty_generics #type_layout_where_clause {
            #consts
        })*

        unsafe impl #type_graph_impl_generics const ::const_type_layout::TypeGraph for
            #ty_name #type_graph_ty_generics #type_graph_where_clause
        {
            fn populate_graph(graph: &mut ::const_type_layout::TypeLayoutGraph<'static>) {
                if graph.insert(&<Self as ::const_type_layout::TypeLayout>::TYPE_LAYOUT) {
                    #(<#inner_types as ::const_type_layout::TypeGraph>::populate_graph(graph);)*
                }
            }
        }
    }.into()
}

struct Attributes {
    reprs: String,
    add_bounds: Vec<syn::WherePredicate>,
    sub_bounds: Vec<String>,
}

fn parse_attributes(
    ty_name: &syn::Ident,
    ty_generics: &syn::TypeGenerics,
    attrs: &[syn::Attribute],
) -> Attributes {
    let mut reprs = Vec::new();

    let mut add_bounds: Vec<syn::WherePredicate> = Vec::new();
    let mut sub_bounds: Vec<String> = vec![quote!(#ty_name #ty_generics).to_string()];

    for attr in attrs {
        if attr.path.is_ident("repr") {
            if let Ok(syn::Meta::List(syn::MetaList { nested, .. })) = attr.parse_meta() {
                for meta in nested {
                    reprs.push(match meta {
                        syn::NestedMeta::Lit(lit) => lit_to_string(&lit),
                        syn::NestedMeta::Meta(meta) => meta_to_string(&meta),
                    });
                }
            } else {
                emit_warning!(
                    attr.span(),
                    "[const-type-layout]: #[repr] attribute is not in meta list format."
                );
            }
        } else if attr.path.is_ident("layout") {
            if let Ok(syn::Meta::List(list)) = attr.parse_meta() {
                for meta in &list.nested {
                    if let syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                        path,
                        lit: syn::Lit::Str(s),
                        ..
                    })) = &meta
                    {
                        if path.is_ident("free") {
                            match syn::parse_str::<syn::Type>(&s.value()) {
                                Ok(ty) => sub_bounds.push(quote!(#ty).to_string()),
                                Err(err) => emit_error!(
                                    s.span(),
                                    "[const-type-layout]: Invalid #[layout(free = \"<type>\")] \
                                     attribute: {}.",
                                    err
                                ),
                            }
                        } else if path.is_ident("bound") {
                            match syn::parse_str(&s.value()) {
                                Ok(bound) => add_bounds.push(bound),
                                Err(err) => emit_error!(
                                    s.span(),
                                    "[const-type-layout]: Invalid #[layout(bound = \
                                     \"<where-predicate>\")] attribute: {}.",
                                    err
                                ),
                            }
                        } else {
                            emit_error!(
                                path.span(),
                                "[const-type-layout]: Unknown attribute, use `free` or `bound`."
                            );
                        }
                    } else {
                        emit_error!(
                            meta.span(),
                            "[const-type-layout]: Expected #[layout(attr = \"value\")] syntax."
                        );
                    }
                }
            } else {
                emit_error!(
                    attr.span(),
                    "[const-type-layout]: Expected #[layout(attr = \"value\")] syntax."
                );
            }
        }
    }

    proc_macro_error::abort_if_dirty();

    reprs.sort();
    reprs.dedup();

    let reprs = reprs
        .into_iter()
        .intersperse(String::from(","))
        .collect::<String>();

    Attributes {
        reprs,
        add_bounds,
        sub_bounds,
    }
}

fn meta_to_string(meta: &syn::Meta) -> String {
    match meta {
        syn::Meta::List(syn::MetaList { path, nested, .. }) => {
            let mut list = nested
                .iter()
                .map(|meta| match meta {
                    syn::NestedMeta::Lit(lit) => lit_to_string(lit),
                    syn::NestedMeta::Meta(meta) => meta_to_string(meta),
                })
                .collect::<Vec<_>>();
            list.sort();
            list.dedup();

            format!(
                "{}({})",
                quote!(#path).to_string(),
                list.into_iter()
                    .intersperse(String::from(","))
                    .collect::<String>()
            )
        },
        syn::Meta::NameValue(syn::MetaNameValue { path, lit, .. }) => {
            format!("{}={}", quote!(#path).to_string(), lit_to_string(lit))
        },
        syn::Meta::Path(path) => quote!(#path).to_string(),
    }
}

fn lit_to_string(lit: &syn::Lit) -> String {
    quote!(#lit).to_string().escape_default().to_string()
}

fn extract_inner_types(data: &syn::Data) -> Vec<&syn::Type> {
    let mut inner_types = Vec::new();

    match data {
        syn::Data::Struct(syn::DataStruct { fields, .. }) => {
            for field in fields {
                inner_types.push(&field.ty);
            }
        },
        syn::Data::Union(syn::DataUnion {
            fields: syn::FieldsNamed { named: fields, .. },
            ..
        }) => {
            for field in fields {
                inner_types.push(&field.ty);
            }
        },
        syn::Data::Enum(syn::DataEnum { variants, .. }) => {
            for variant in variants {
                for field in &variant.fields {
                    inner_types.push(&field.ty);
                }
            }
        },
    }

    inner_types
}

struct Generics {
    type_layout_input_generics: syn::Generics,
    type_graph_input_generics: syn::Generics,
}

fn generate_generics(
    ty_name: &syn::Ident,
    ty_generics: &syn::TypeGenerics,
    generics: &syn::Generics,
    is_enum: bool,
    inner_types: &[&syn::Type],
    add_bounds: Vec<syn::WherePredicate>,
    sub_bounds: &[String],
) -> Generics {
    let mut type_layout_input_generics = generics.clone();

    if is_enum {
        type_layout_input_generics
            .make_where_clause()
            .predicates
            .push(syn::parse_quote! {
                [u8; ::core::mem::size_of::<::core::mem::Discriminant<#ty_name #ty_generics>>()]:
            });
    }

    let mut type_graph_input_generics = generics.clone();

    if is_enum {
        type_graph_input_generics
            .make_where_clause()
            .predicates
            .push(syn::parse_quote! {
                [u8; ::core::mem::size_of::<::core::mem::Discriminant<#ty_name #ty_generics>>()]:
            });
    }

    for ty in inner_types {
        if !sub_bounds.contains(&quote!(#ty).to_string()) {
            type_layout_input_generics
                .make_where_clause()
                .predicates
                .push(syn::parse_quote! {
                    #ty: ::const_type_layout::TypeLayout
                });

            type_graph_input_generics
                .make_where_clause()
                .predicates
                .push(syn::parse_quote! {
                    #ty: ~const ::const_type_layout::TypeGraph
                });
        }
    }

    for bound in add_bounds {
        type_layout_input_generics
            .make_where_clause()
            .predicates
            .push(bound.clone());
        type_graph_input_generics
            .make_where_clause()
            .predicates
            .push(bound);
    }

    Generics {
        type_layout_input_generics,
        type_graph_input_generics,
    }
}

fn layout_of_type(
    ty_name: &syn::Ident,
    ty_generics: &syn::TypeGenerics,
    data: &syn::Data,
    reprs: &str,
    consts: &mut Vec<proc_macro2::TokenStream>,
) -> proc_macro2::TokenStream {
    match data {
        syn::Data::Struct(data) => {
            let fields = quote_fields(
                ty_name,
                None,
                &quote_structlike_fields(ty_name, ty_generics, &data.fields, false),
                consts,
            );

            quote! {
                ::const_type_layout::TypeStructure::Struct { repr: #reprs, fields: #fields }
            }
        },
        syn::Data::Enum(r#enum) => {
            let variants = quote_variants(
                ty_name,
                &quote_enum_variants(ty_name, ty_generics, r#enum, consts),
                consts,
            );

            quote! {
                ::const_type_layout::TypeStructure::Enum { repr: #reprs, variants: #variants }
            }
        },
        syn::Data::Union(union) => {
            let fields = quote_fields(
                ty_name,
                None,
                &quote_structlike_fields(
                    ty_name,
                    ty_generics,
                    &syn::Fields::Named(union.fields.clone()),
                    true,
                ),
                consts,
            );

            quote! {
                ::const_type_layout::TypeStructure::Union { repr: #reprs, fields: #fields }
            }
        },
    }
}

fn quote_structlike_fields(
    ty_name: &syn::Ident,
    ty_generics: &syn::TypeGenerics,
    fields: &syn::Fields,
    is_union: bool,
) -> Vec<proc_macro2::TokenStream> {
    match fields {
        syn::Fields::Named(fields) => fields
            .named
            .iter()
            .map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_name_str = Literal::string(&field_name.to_string());
                let field_ty = &field.ty;
                let field_offset =
                    quote_structlike_field_offset(ty_name, ty_generics, &field_name, is_union);

                quote_spanned! { field.span() =>
                    ::const_type_layout::Field {
                        name: #field_name_str,
                        offset: { #field_offset },
                        ty: ::core::any::type_name::<#field_ty>(),
                    }
                }
            })
            .collect(),
        syn::Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(field_index, field)| {
                let field_name = syn::Index::from(field_index);
                let field_name_str = Literal::string(&field_index.to_string());
                let field_ty = &field.ty;
                let field_offset =
                    quote_structlike_field_offset(ty_name, ty_generics, &field_name, is_union);

                quote_spanned! { field.span() =>
                    ::const_type_layout::Field {
                        name: #field_name_str,
                        offset: { #field_offset },
                        ty: ::core::any::type_name::<#field_ty>(),
                    }
                }
            })
            .collect(),
        syn::Fields::Unit => vec![],
    }
}

fn quote_structlike_field_offset(
    ty_name: &syn::Ident,
    ty_generics: &syn::TypeGenerics,
    field_name: &impl quote::ToTokens,
    is_union: bool,
) -> proc_macro2::TokenStream {
    let extra_fields = if is_union { quote!() } else { quote!(..) };

    quote! {
        let uninit = ::core::mem::MaybeUninit::<#ty_name #ty_generics>::uninit();
        let base_ptr: *const #ty_name #ty_generics = uninit.as_ptr();

        #[allow(clippy::unneeded_field_pattern)]
        let #ty_name { #field_name: _, #extra_fields }: #ty_name #ty_generics;

        #[allow(unused_unsafe)]
        let field_ptr = unsafe {
            ::core::ptr::addr_of!((*base_ptr).#field_name)
        };

        #[allow(clippy::cast_sign_loss)]
        unsafe { field_ptr.cast::<u8>().offset_from(base_ptr.cast()) as usize }
    }
}

fn quote_fields(
    ty_name: &syn::Ident,
    qualifier: Option<&syn::Ident>,
    fields: &[proc_macro2::TokenStream],
    consts: &mut Vec<proc_macro2::TokenStream>,
) -> proc_macro2::TokenStream {
    let fields_len = fields.len();

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
        const #ident: &'static [::const_type_layout::Field<'static>; #fields_len] = &[#(#fields),*];
    });

    quote! { Self :: #ident }
}

fn quote_enum_variants(
    ty_name: &syn::Ident,
    ty_generics: &syn::TypeGenerics,
    r#enum: &syn::DataEnum,
    consts: &mut Vec<proc_macro2::TokenStream>,
) -> Vec<proc_macro2::TokenStream> {
    r#enum
        .variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.ident;
            let variant_name_str = Literal::string(&variant_name.to_string());

            let variant_constructor =
                quote_variant_constructor(ty_name, variant_name, &variant.fields);
            let variant_destructor =
                quote_variant_destructor(ty_name, variant_name, &variant.fields);

            let fields = quote_fields(
                ty_name,
                Some(variant_name),
                &quote_variant_fields(
                    ty_name,
                    ty_generics,
                    &variant.fields,
                    &variant_constructor,
                    &variant_destructor,
                ),
                consts,
            );

            let discriminant_bytes = quote_discriminant_bytes(
                ty_name,
                ty_generics,
                variant_name,
                &variant_constructor,
                consts,
            );

            quote! {
                ::const_type_layout::Variant {
                    name: #variant_name_str,
                    discriminant: ::const_type_layout::Discriminant {
                        big_endian_bytes: &#discriminant_bytes,
                    },
                    fields: #fields,
                }
            }
        })
        .collect::<Vec<_>>()
}

fn quote_variant_constructor(
    ty_name: &syn::Ident,
    variant_name: &syn::Ident,
    variant_fields: &syn::Fields,
) -> proc_macro2::TokenStream {
    match variant_fields {
        syn::Fields::Unit => quote! { #ty_name::#variant_name },
        syn::Fields::Unnamed(fields) => {
            let initialisers = fields.unnamed.iter().map(|field| {
                let field_ty = &field.ty;

                quote! { unsafe {
                    let mut value: ::core::mem::MaybeUninit<#field_ty> = ::core::mem::MaybeUninit::uninit();

                    let mut i = 0;
                    while i < core::mem::size_of::<#field_ty>() {
                        *value.as_mut_ptr().cast::<u8>().add(i) = 0xFF_u8;
                        i += 1;
                    }

                    value.assume_init()
                } }
            }).collect::<Vec<_>>();

            quote! { #ty_name::#variant_name(#(#initialisers),*) }
        },
        syn::Fields::Named(fields) => {
            let initialisers = fields.named.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_ty = &field.ty;

                quote! { #field_name: unsafe {
                    let mut value: ::core::mem::MaybeUninit<#field_ty> = ::core::mem::MaybeUninit::uninit();

                    let mut i = 0;
                    while i < core::mem::size_of::<#field_ty>() {
                        *value.as_mut_ptr().cast::<u8>().add(i) = 0xFF_u8;
                        i += 1;
                    }

                    value.assume_init()
                } }
            }).collect::<Vec<_>>();

            quote! { #ty_name::#variant_name { #(#initialisers),* } }
        },
    }
}

fn quote_variant_destructor(
    ty_name: &syn::Ident,
    variant_name: &syn::Ident,
    variant_fields: &syn::Fields,
) -> proc_macro2::TokenStream {
    match variant_fields {
        syn::Fields::Unit => quote! { #ty_name::#variant_name },
        syn::Fields::Unnamed(fields) => {
            let destructors = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(field_index, _)| {
                    let field_name = quote::format_ident!("__self_{}", field_index);

                    quote! { #field_name }
                })
                .collect::<Vec<_>>();

            quote! { #ty_name::#variant_name(#(#destructors),*) }
        },
        syn::Fields::Named(fields) => {
            let destructors = fields
                .named
                .iter()
                .map(|field| {
                    let field_name = field.ident.as_ref().unwrap();

                    quote! { #field_name }
                })
                .collect::<Vec<_>>();

            quote! { #ty_name::#variant_name { #(#destructors),* } }
        },
    }
}

fn quote_variant_fields(
    ty_name: &syn::Ident,
    ty_generics: &syn::TypeGenerics,
    variant_fields: &syn::Fields,
    variant_constructor: &proc_macro2::TokenStream,
    variant_destructor: &proc_macro2::TokenStream,
) -> Vec<proc_macro2::TokenStream> {
    match variant_fields {
        syn::Fields::Named(fields) => {
            fields.named.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_name_str = Literal::string(&field_name.to_string());
                let field_ty = &field.ty;

                quote_spanned! { field.span() =>
                    ::const_type_layout::Field {
                        name: #field_name_str,
                        offset: {
                            let __variant_base: ::core::mem::MaybeUninit<#ty_name #ty_generics> = ::core::mem::MaybeUninit::new(#variant_constructor);

                            #[allow(
                                unused_variables, unreachable_patterns, clippy::cast_sign_loss,
                                clippy::match_wildcard_for_single_variants
                            )]
                            match unsafe { __variant_base.assume_init_ref() } {
                                #variant_destructor => unsafe {
                                    (#field_name as *const #field_ty).cast::<u8>().offset_from(__variant_base.as_ptr().cast()) as usize
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
                    ::const_type_layout::Field {
                        name: #field_name_str,
                        offset: {
                            let __variant_base: ::core::mem::MaybeUninit<#ty_name #ty_generics> = ::core::mem::MaybeUninit::new(#variant_constructor);

                            #[allow(
                                unused_variables, unreachable_patterns, clippy::cast_sign_loss,
                                clippy::match_wildcard_for_single_variants
                            )]
                            match unsafe { __variant_base.assume_init_ref() } {
                                #variant_destructor => unsafe {
                                    (#field_name as *const #field_ty).cast::<u8>().offset_from(__variant_base.as_ptr().cast()) as usize
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
    }
}

fn quote_discriminant_bytes(
    ty_name: &syn::Ident,
    ty_generics: &syn::TypeGenerics,
    variant_name: &syn::Ident,
    variant_constructor: &proc_macro2::TokenStream,
    consts: &mut Vec<proc_macro2::TokenStream>,
) -> proc_macro2::TokenStream {
    let ident = syn::Ident::new(
        &format!("__{}_{}_discriminant", ty_name, variant_name).to_uppercase(),
        ty_name.span(),
    );

    consts.push(quote! {
        const #ident: [u8; ::core::mem::size_of::<::core::mem::Discriminant<#ty_name #ty_generics>>()] = unsafe {
            let variant: ::core::mem::MaybeUninit<#ty_name #ty_generics> = ::core::mem::MaybeUninit::new(#variant_constructor);

            let system_endian_bytes: [u8; ::core::mem::size_of::<::core::mem::Discriminant<#ty_name #ty_generics>>()] = ::core::mem::transmute(
                ::core::mem::discriminant(variant.assume_init_ref())
            );

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

    quote! { Self :: #ident }
}

fn quote_variants(
    ty_name: &syn::Ident,
    variants: &[proc_macro2::TokenStream],
    consts: &mut Vec<proc_macro2::TokenStream>,
) -> proc_macro2::TokenStream {
    let variants_len = variants.len();

    let ident = syn::Ident::new(
        &format!("__{}_variants", ty_name).to_uppercase(),
        ty_name.span(),
    );

    consts.push(quote! {
        const #ident: &'static [::const_type_layout::Variant<'static>; #variants_len] = &[#(#variants),*];
    });

    quote! { Self :: #ident }
}
