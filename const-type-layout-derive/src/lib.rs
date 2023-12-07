#![deny(clippy::pedantic)]
#![feature(cfg_version)]
#![feature(iter_intersperse)]
#![cfg_attr(not(version("1.66.0")), feature(let_else))]

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

    let mut type_params = input
        .generics
        .type_params()
        .map(|param| &param.ident)
        .collect::<Vec<_>>();

    let Attributes {
        reprs,
        extra_bounds,
        ground,
        crate_path,
    } = parse_attributes(&input.attrs, &mut type_params, &input.data);

    let layout = layout_of_type(&crate_path, &ty_name, &ty_generics, &input.data, &reprs);
    let uninit = uninit_for_type(&crate_path, &ty_name, &input.data, &ground);

    let inner_types = extract_inner_types(&input.data);

    let Generics {
        type_layout_input_generics,
        type_set_input_generics,
    } = generate_generics(
        &crate_path,
        &ty_name,
        &ty_generics,
        &input.generics,
        matches!(input.data, syn::Data::Enum(_)),
        &extra_bounds,
        &type_params,
    );
    let (type_layout_impl_generics, type_layout_ty_generics, type_layout_where_clause) =
        type_layout_input_generics.split_for_impl();
    let (type_set_impl_generics, type_set_ty_generics, type_set_where_clause) =
        type_set_input_generics.split_for_impl();

    quote! {
        unsafe impl #type_layout_impl_generics const #crate_path::TypeLayout for
            #ty_name #type_layout_ty_generics #type_layout_where_clause
        {
            const TYPE_LAYOUT: #crate_path::TypeLayoutInfo<'static> = {
                #crate_path::TypeLayoutInfo {
                    name: ::core::any::type_name::<Self>(),
                    size: ::core::mem::size_of::<Self>(),
                    alignment: ::core::mem::align_of::<Self>(),
                    structure: #layout,
                }
            };

            unsafe fn uninit() -> #crate_path::MaybeUninhabited<
                ::core::mem::MaybeUninit<Self>
            > {
                #uninit
            }
        }

        unsafe impl #type_set_impl_generics #crate_path::typeset::ComputeTypeSet for
            #ty_name #type_set_ty_generics #type_set_where_clause
        {
            type Output<__TypeSetRest: #crate_path::typeset::ExpandTypeSet> =
                #crate_path::typeset::Set<Self, #crate_path::typeset::tset![
                    #(#inner_types,)* .. @ __TypeSetRest
                ]>;
        }
    }
    .into()
}

struct Attributes {
    reprs: String,
    extra_bounds: Vec<syn::WherePredicate>,
    ground: Vec<syn::Ident>,
    crate_path: syn::Path,
}

#[allow(clippy::too_many_lines)]
fn parse_attributes(
    attrs: &[syn::Attribute],
    type_params: &mut Vec<&syn::Ident>,
    data: &syn::Data,
) -> Attributes {
    // Could parse based on https://github.com/rust-lang/rust/blob/d13e8dd41d44a73664943169d5b7fe39b22c449f/compiler/rustc_attr/src/builtin.rs#L772-L781 instead
    let mut reprs = Vec::new();

    let mut extra_bounds: Vec<syn::WherePredicate> = Vec::new();

    let mut ground = match data {
        syn::Data::Struct(_) => Vec::new(),
        syn::Data::Enum(syn::DataEnum { variants, .. }) => {
            let mut ground = Vec::with_capacity(variants.len());

            for variant in variants {
                if matches!(variant.fields, syn::Fields::Unit) {
                    ground.push(variant.ident.clone());
                }
            }

            for variant in variants {
                if !matches!(variant.fields, syn::Fields::Unit) {
                    ground.push(variant.ident.clone());
                }
            }

            ground
        },
        syn::Data::Union(syn::DataUnion {
            fields: syn::FieldsNamed { named: fields, .. },
            ..
        }) => fields
            .iter()
            .map(|field| field.ident.clone().unwrap())
            .collect(),
    };
    let mut groundier = Vec::with_capacity(ground.len());

    let mut crate_path = None;

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
                            match syn::parse_str::<syn::Ident>(&s.value()) {
                                Ok(param) => {
                                    if let Some(i) = type_params.iter().position(|ty| **ty == param)
                                    {
                                        type_params.swap_remove(i);
                                    } else {
                                        emit_error!(
                                            s.span(),
                                            "[const-type-layout]: Invalid #[layout(free)] \
                                             attribute: \"{}\" is either not a type parameter or \
                                             has already been freed (duplicate attribute).",
                                            param,
                                        );
                                    }
                                },
                                Err(err) => emit_error!(
                                    s.span(),
                                    "[const-type-layout]: Invalid #[layout(free = \"<type>\")] \
                                     attribute: {}.",
                                    err
                                ),
                            }
                        } else if path.is_ident("bound") {
                            match syn::parse_str(&s.value()) {
                                Ok(bound) => extra_bounds.push(bound),
                                Err(err) => emit_error!(
                                    s.span(),
                                    "[const-type-layout]: Invalid #[layout(bound = \
                                     \"<where-predicate>\")] attribute: {}.",
                                    err
                                ),
                            }
                        } else if path.is_ident("ground") {
                            match syn::parse_str(&s.value()) {
                                Ok(g) => match data {
                                    syn::Data::Struct(_) => emit_error!(
                                        path.span(),
                                        "[const-type-layout]: Invalid #[layout(ground)] \
                                         attribute: structs do not have a ground layout."
                                    ),
                                    syn::Data::Union(_) | syn::Data::Enum(_) => {
                                        let g: syn::Ident = g;

                                        if let Some(i) = ground.iter().position(|e| e == &g) {
                                            let g = ground.remove(i);
                                            groundier.push(g);
                                        } else if groundier.contains(&g) {
                                            emit_error!(
                                                path.span(),
                                                "[const-type-layout]: Duplicate #[layout(ground = \
                                                 \"{}\")] attribute.",
                                                g
                                            );
                                        } else {
                                            emit_error!(
                                                path.span(),
                                                "[const-type-layout]: Invalid #[layout(ground)] \
                                                 attribute: \"{}\" is not a {} in this {}.",
                                                g,
                                                match data {
                                                    syn::Data::Enum(_) => "variant",
                                                    syn::Data::Struct(_) | syn::Data::Union(_) =>
                                                        "field",
                                                },
                                                match data {
                                                    syn::Data::Enum(_) => "enum",
                                                    syn::Data::Struct(_) | syn::Data::Union(_) =>
                                                        "union",
                                                },
                                            );
                                        }
                                    },
                                },
                                Err(err) => emit_error!(
                                    s.span(),
                                    "[const-type-layout]: Invalid #[layout(ground = \"{}\")] \
                                     attribute: {}.",
                                    match data {
                                        syn::Data::Enum(_) => "variant",
                                        syn::Data::Struct(_) | syn::Data::Union(_) => "field",
                                    },
                                    err
                                ),
                            }
                        } else if path.is_ident("crate") {
                            match syn::parse_str::<syn::Path>(&s.value()) {
                                Ok(new_crate_path) => {
                                    if crate_path.is_none() {
                                        crate_path = Some(
                                            syn::parse_quote_spanned! { s.span() => #new_crate_path },
                                        );
                                    } else {
                                        emit_error!(
                                            s.span(),
                                            "[const-type-layout]: Duplicate #[layout(crate)] \
                                             attribute: the crate path for `const-type-layout` \
                                             can only be set once per `derive`.",
                                        );
                                    }
                                },
                                Err(err) => emit_error!(
                                    s.span(),
                                    "[const-type-layout]: Invalid #[layout(crate = \
                                     \"<crate-path>\")] attribute: {}.",
                                    err
                                ),
                            }
                        } else {
                            emit_error!(
                                path.span(),
                                "[const-type-layout]: Unknown attribute, use `bound`, `crate`, \
                                 `free`, or `ground`."
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

    groundier.extend(ground);

    Attributes {
        reprs,
        extra_bounds,
        ground: groundier,
        crate_path: crate_path.unwrap_or_else(|| syn::parse_quote!(::const_type_layout)),
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
                quote!(#path),
                list.into_iter()
                    .intersperse(String::from(","))
                    .collect::<String>()
            )
        },
        syn::Meta::NameValue(syn::MetaNameValue { path, lit, .. }) => {
            format!("{}={}", quote!(#path), lit_to_string(lit))
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
    type_set_input_generics: syn::Generics,
}

fn generate_generics(
    crate_path: &syn::Path,
    ty_name: &syn::Ident,
    ty_generics: &syn::TypeGenerics,
    generics: &syn::Generics,
    is_enum: bool,
    extra_bounds: &[syn::WherePredicate],
    type_params: &[&syn::Ident],
) -> Generics {
    let mut type_layout_input_generics = generics.clone();
    let mut type_set_input_generics = generics.clone();

    if is_enum {
        type_layout_input_generics
            .make_where_clause()
            .predicates
            .push(syn::parse_quote! {
                [u8; ::core::mem::size_of::<::core::mem::Discriminant<#ty_name #ty_generics>>()]:
            });

        type_set_input_generics
            .make_where_clause()
            .predicates
            .push(syn::parse_quote! {
                [u8; ::core::mem::size_of::<::core::mem::Discriminant<#ty_name #ty_generics>>()]:
            });
    }

    for ty in type_params {
        type_layout_input_generics
            .make_where_clause()
            .predicates
            .push(syn::parse_quote! {
                #ty: ~const #crate_path::TypeLayout
            });

        type_set_input_generics
            .make_where_clause()
            .predicates
            .push(syn::parse_quote! {
                #ty: #crate_path::typeset::ComputeTypeSet
            });
    }

    for bound in extra_bounds {
        type_layout_input_generics
            .make_where_clause()
            .predicates
            .push(bound.clone());

        type_set_input_generics
            .make_where_clause()
            .predicates
            .push(bound.clone());
    }

    Generics {
        type_layout_input_generics,
        type_set_input_generics,
    }
}

fn layout_of_type(
    crate_path: &syn::Path,
    ty_name: &syn::Ident,
    ty_generics: &syn::TypeGenerics,
    data: &syn::Data,
    reprs: &str,
) -> proc_macro2::TokenStream {
    match data {
        syn::Data::Struct(data) => {
            let fields =
                quote_structlike_fields(crate_path, ty_name, ty_generics, &data.fields, false);

            quote! {
                #crate_path::TypeStructure::Struct { repr: #reprs, fields: &[#(#fields),*] }
            }
        },
        syn::Data::Enum(r#enum) => {
            let variants = quote_enum_variants(crate_path, ty_name, ty_generics, r#enum);

            quote! {
                #crate_path::TypeStructure::Enum { repr: #reprs, variants: &[#(#variants),*] }
            }
        },
        syn::Data::Union(union) => {
            let fields = quote_structlike_fields(
                crate_path,
                ty_name,
                ty_generics,
                &syn::Fields::Named(union.fields.clone()),
                true,
            );

            quote! {
                #crate_path::TypeStructure::Union { repr: #reprs, fields: &[#(#fields),*] }
            }
        },
    }
}

fn quote_structlike_fields(
    crate_path: &syn::Path,
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
                let field_offset = quote_structlike_field_offset(
                    crate_path,
                    ty_name,
                    ty_generics,
                    &field_name,
                    is_union,
                );

                quote_spanned! { field.span() =>
                    #crate_path::Field {
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
                let field_offset = quote_structlike_field_offset(
                    crate_path,
                    ty_name,
                    ty_generics,
                    &field_name,
                    is_union,
                );

                quote_spanned! { field.span() =>
                    #crate_path::Field {
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
    crate_path: &syn::Path,
    ty_name: &syn::Ident,
    ty_generics: &syn::TypeGenerics,
    field_name: &impl quote::ToTokens,
    is_union: bool,
) -> proc_macro2::TokenStream {
    let extra_fields = if is_union { quote!() } else { quote!(..) };

    quote! {
        #crate_path::struct_field_offset!(#ty_name => #ty_name #ty_generics => (*base_ptr).#field_name => #extra_fields)
    }
}

fn quote_enum_variants(
    crate_path: &syn::Path,
    ty_name: &syn::Ident,
    ty_generics: &syn::TypeGenerics,
    r#enum: &syn::DataEnum,
) -> Vec<proc_macro2::TokenStream> {
    r#enum
        .variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.ident;
            let variant_name_str = Literal::string(&variant_name.to_string());

            let fields = quote_variant_fields(
                crate_path,
                ty_name,
                ty_generics,
                variant_name,
                &variant.fields,
            );

            let discriminant = quote_discriminant(
                crate_path,
                ty_name,
                ty_generics,
                variant_name,
                &variant.fields,
            );

            quote! {
                #crate_path::Variant {
                    name: #variant_name_str,
                    discriminant: #discriminant,
                    fields: &[#(#fields),*],
                }
            }
        })
        .collect::<Vec<_>>()
}

fn quote_variant_fields(
    crate_path: &syn::Path,
    ty_name: &syn::Ident,
    ty_generics: &syn::TypeGenerics,
    variant_name: &syn::Ident,
    variant_fields: &syn::Fields,
) -> Vec<proc_macro2::TokenStream> {
    match variant_fields {
        syn::Fields::Named(syn::FieldsNamed { named: fields, .. }) => {
            let field_descriptors = fields
                .pairs()
                .map(|pair| {
                    let syn::Field {
                        ident: field_name,
                        colon_token: field_colon,
                        ty: field_ty,
                        ..
                    } = pair.value();
                    let field_comma = pair.punct();

                    quote!(#field_name #field_colon #field_ty #field_comma)
                })
                .collect::<Vec<_>>();

            fields.iter().map(|field| {
                let field_name_str = Literal::string(&field.ident.as_ref().unwrap().to_string());
                let field_index = &field.ident;
                let field_ty = &field.ty;

                quote_spanned! { field.span() =>
                    #crate_path::Field {
                        name: #field_name_str,
                        offset: #crate_path::struct_variant_field_offset!(
                            #ty_name => #ty_name #ty_generics => #variant_name { #(#field_descriptors)* } => #field_index
                        ),
                        ty: ::core::any::type_name::<#field_ty>(),
                    }
                }
            }).collect()
        },
        syn::Fields::Unnamed(syn::FieldsUnnamed {
            unnamed: fields, ..
        }) => {
            let field_descriptors = fields
                .pairs()
                .enumerate()
                .map(|(i, pair)| {
                    let syn::Field { ty: field_ty, .. } = pair.value();
                    let field_name = quote::format_ident!("f_{}", i);
                    let field_comma = pair.punct();

                    quote!(#field_name: #field_ty #field_comma)
                })
                .collect::<Vec<_>>();

            fields.iter().enumerate().map(|(field_index, field)| {
                let field_name_str = Literal::string(&field_index.to_string());
                let field_index = syn::Index::from(field_index);
                let field_ty = &field.ty;

                quote_spanned! { field.span() =>
                    #crate_path::Field {
                        name: #field_name_str,
                        offset: #crate_path::struct_variant_field_offset!(
                            #ty_name => #ty_name #ty_generics => #variant_name(#(#field_descriptors)*) => #field_index
                        ),
                        ty: ::core::any::type_name::<#field_ty>(),
                    }
                }
            }).collect()
        },
        syn::Fields::Unit => vec![],
    }
}

fn quote_discriminant(
    crate_path: &syn::Path,
    ty_name: &syn::Ident,
    ty_generics: &syn::TypeGenerics,
    variant_name: &syn::Ident,
    variant_fields: &syn::Fields,
) -> proc_macro2::TokenStream {
    let variant_descriptor = match variant_fields {
        syn::Fields::Named(syn::FieldsNamed { named: fields, .. }) => {
            let field_descriptors = fields
                .pairs()
                .map(|pair| {
                    let syn::Field {
                        ident: field_name,
                        colon_token: field_colon,
                        ty: field_ty,
                        ..
                    } = pair.value();
                    let field_comma = pair.punct();

                    quote!(#field_name #field_colon #field_ty #field_comma)
                })
                .collect::<Vec<_>>();

            quote!(#variant_name { #(#field_descriptors)* })
        },
        syn::Fields::Unnamed(syn::FieldsUnnamed {
            unnamed: fields, ..
        }) => {
            let field_descriptors = fields
                .pairs()
                .enumerate()
                .map(|(i, pair)| {
                    let syn::Field { ty: field_ty, .. } = pair.value();
                    let field_name = quote::format_ident!("f_{}", i);
                    let field_comma = pair.punct();

                    quote!(#field_name: #field_ty #field_comma)
                })
                .collect::<Vec<_>>();

            quote!(#variant_name(#(#field_descriptors)*))
        },
        syn::Fields::Unit => quote!(#variant_name),
    };

    quote! {
        #crate_path::struct_variant_discriminant!(
            #ty_name => #ty_name #ty_generics => #variant_descriptor
        )
    }
}

#[allow(clippy::too_many_lines)]
fn uninit_for_type(
    crate_path: &syn::Path,
    ty_name: &syn::Ident,
    data: &syn::Data,
    ground: &[syn::Ident],
) -> proc_macro2::TokenStream {
    match data {
        syn::Data::Struct(data) => {
            // Structs are uninhabited if any of their fields in uninhabited

            let fields = match &data.fields {
                syn::Fields::Named(syn::FieldsNamed { named: fields, .. })
                | syn::Fields::Unnamed(syn::FieldsUnnamed {
                    unnamed: fields, ..
                }) => fields,
                syn::Fields::Unit => {
                    return quote! {
                        #crate_path::MaybeUninhabited::Inhabited(
                            ::core::mem::MaybeUninit::new(#ty_name)
                        )
                    }
                },
            };

            let field_names = fields
                .iter()
                .enumerate()
                .map(|(i, field)| match &field.ident {
                    Some(name) => name.clone(),
                    None => quote::format_ident!("f_{}", i),
                })
                .collect::<Vec<_>>();

            let field_initialisers = fields
                .iter()
                .map(|syn::Field { ty: field_ty, .. }| {
                    quote! {
                        <
                            #field_ty as #crate_path::TypeLayout
                        >::uninit()
                    }
                })
                .collect::<Vec<_>>();

            let struct_initialiser = if let syn::Fields::Named(_) = &data.fields {
                quote!(#ty_name { #(#field_names: #field_names.assume_init()),* })
            } else {
                quote!(#ty_name ( #(#field_names.assume_init()),* ))
            };

            quote! {
                if let (
                    #(#crate_path::MaybeUninhabited::Inhabited(#field_names)),*
                ) = (
                    #(#field_initialisers),*
                ) {
                    #crate_path::MaybeUninhabited::Inhabited(
                        ::core::mem::MaybeUninit::new(
                            #struct_initialiser
                        )
                    )
                } else {
                    #crate_path::MaybeUninhabited::Uninhabited
                }
            }
        },
        syn::Data::Enum(syn::DataEnum { variants, .. }) => {
            // Enums are uninhabited if
            //  (a) they have no variants
            //  (b) all their variants are uninhabited
            //    (1) unit variants are always inhabited
            //    (2) tuple and struct variants are uninhabited
            //        if any of their fields are uninhabited

            let variant_initialisers = ground.iter().filter_map(|g| variants.iter().find(|v| &v.ident == g)).map(|syn::Variant {
                ident: variant_name,
                fields: variant_fields,
                ..
            }| {
                let fields = match variant_fields {
                    syn::Fields::Named(syn::FieldsNamed { named: fields, .. })
                    | syn::Fields::Unnamed(syn::FieldsUnnamed {
                        unnamed: fields, ..
                    }) => fields,
                    syn::Fields::Unit => return Err(quote! {
                        #crate_path::MaybeUninhabited::Inhabited(
                            ::core::mem::MaybeUninit::new(
                                #ty_name :: #variant_name
                            )
                        )
                    }),
                };

                let field_names = fields.iter().enumerate().map(|(i, field)| match &field.ident {
                    Some(name) => name.clone(),
                    None => quote::format_ident!("f_{}", i),
                }).collect::<Vec<_>>();

                let field_initialisers = fields
                    .iter()
                    .map(|syn::Field {
                        ty: field_ty,
                        ..
                    }| {
                        quote! {
                            <
                                #field_ty as #crate_path::TypeLayout
                            >::uninit()
                        }
                    })
                    .collect::<Vec<_>>();

                let variant_initialiser = if let syn::Fields::Named(_) = variant_fields {
                    quote!(#ty_name :: #variant_name { #(#field_names: #field_names.assume_init()),* })
                } else {
                    quote!(#ty_name :: #variant_name ( #(#field_names.assume_init()),* ))
                };

                Ok(quote!{
                    if let (
                        #(#crate_path::MaybeUninhabited::Inhabited(#field_names)),*
                    ) = (
                        #(#field_initialisers),*
                    ) {
                        return #crate_path::MaybeUninhabited::Inhabited(
                            ::core::mem::MaybeUninit::new(
                                #variant_initialiser
                            )
                        );
                    }
                })
            }).collect::<Result<Vec<proc_macro2::TokenStream>, proc_macro2::TokenStream>>();

            match variant_initialisers {
                Ok(variant_initialisers) => quote! {
                    #(
                        #variant_initialisers
                    )*

                    #crate_path::MaybeUninhabited::Uninhabited
                },
                Err(unit_variant_initialiser) => unit_variant_initialiser,
            }
        },
        syn::Data::Union(syn::DataUnion {
            fields: syn::FieldsNamed { named: fields, .. },
            ..
        }) => {
            // Unions are uninhabited if all fields are uninhabited

            let (field_names, field_initialisers) = ground
                .iter()
                .filter_map(|g| fields.iter().find(|f| f.ident.as_ref() == Some(g)))
                .map(
                    |syn::Field {
                         ident: field_name,
                         ty: field_ty,
                         ..
                     }| {
                        (
                            field_name,
                            quote! {
                                <
                                    #field_ty as #crate_path::TypeLayout
                                >::uninit()
                            },
                        )
                    },
                )
                .unzip::<_, _, Vec<_>, Vec<_>>();

            quote! {
                #(
                    if let #crate_path::MaybeUninhabited::Inhabited(
                        #field_names
                    ) = #field_initialisers {
                        return #crate_path::MaybeUninhabited::Inhabited(
                            ::core::mem::MaybeUninit::new(
                                #ty_name { #field_names: #field_names.assume_init() }
                            )
                        );
                    }
                )*

                #crate_path::MaybeUninhabited::Uninhabited
            }
        },
    }
}
