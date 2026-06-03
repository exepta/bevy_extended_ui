use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    token::Eq,
    FnArg, GenericArgument, Item, ItemEnum, ItemFn, ItemStruct, ItemUse, LitStr, PathArguments,
    Result, Type, TypePath, UseTree,
};

/// Parsed attribute arguments for the `html_fn` macro.
struct HtmlFnAttr {
    name: LitStr,
}

impl Parse for HtmlFnAttr {
    /// Parses the macro attribute input into `HtmlFnAttr`.
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Eq) {
            let _eq: Eq = input.parse()?;
            Ok(Self {
                name: input.parse()?,
            })
        } else {
            Ok(Self {
                name: input.parse()?,
            })
        }
    }
}

/// Registers a function as an HTML event handler.
#[proc_macro_attribute]
pub fn html_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    let HtmlFnAttr { name } = parse_macro_input!(attr as HtmlFnAttr);
    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_ident = input_fn.sig.ident.clone();
    let builder_ident = format_ident!("__html_fn_build_{}", fn_ident);
    let (event_variant, event_type) = match extract_event_type(&input_fn) {
        Ok(Some((variant, ty))) => (variant, ty),
        Ok(None) => (
            format_ident!("HtmlEvent"),
            syn::parse_quote!(bevy_extended_ui::html::HtmlEvent),
        ),
        Err(err) => {
            let err_tokens = err.to_compile_error();
            return TokenStream::from(quote! { #err_tokens });
        }
    };
    let expanded = quote! {
        #input_fn

        #[doc(hidden)]
        fn #builder_ident(world: &mut bevy::prelude::World) -> bevy::ecs::system::SystemId<bevy::prelude::In<#event_type>, ()> {
            world.register_system(#fn_ident)
        }

        bevy_extended_ui::html::inventory::submit! {
            bevy_extended_ui::html::HtmlFnRegistration::#event_variant {
                name: #name,
                build: #builder_ident,
            }
        }
    };

    expanded.into()
}

/// Registers a component constructor that runs once during Bevy `Startup`.
#[proc_macro_attribute]
pub fn component_init(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_ident = input_fn.sig.ident.clone();
    let builder_ident = format_ident!("__component_init_build_{}", fn_ident);
    let shared_registrations =
        expand_resource_shared_registrations("component_init", &fn_ident, &input_fn);

    quote! {
        #input_fn

        #shared_registrations

        #[doc(hidden)]
        fn #builder_ident(world: &mut bevy::prelude::World) -> bevy::ecs::system::SystemId<(), ()> {
            world.register_system(#fn_ident)
        }

        bevy_extended_ui::html::inventory::submit! {
            bevy_extended_ui::html::ComponentInitRegistration {
                name: stringify!(#fn_ident),
                build: #builder_ident,
            }
        }
    }
    .into()
}

fn expand_resource_shared_registrations(
    capture_prefix: &str,
    fn_ident: &syn::Ident,
    input_fn: &ItemFn,
) -> proc_macro2::TokenStream {
    let mut registrations = Vec::new();
    let mut seen = Vec::<String>::new();

    for arg in &input_fn.sig.inputs {
        let FnArg::Typed(pat_type) = arg else {
            continue;
        };
        let Some(resource_ty) = extract_read_resource_type(&pat_type.ty) else {
            continue;
        };
        let Some(shared_name) = simple_type_name_from_type(&resource_ty) else {
            continue;
        };

        let type_key = quote!(#resource_ty).to_string();
        if seen.iter().any(|seen| seen == &type_key) {
            continue;
        }
        seen.push(type_key.clone());

        let default_alias = to_default_alias(&shared_name);
        let name_lit = LitStr::new(&shared_name, fn_ident.span());
        let alias_lit = LitStr::new(&default_alias, fn_ident.span());
        let capture_ident = format_ident!(
            "__{}_shared_capture_{}_{}",
            capture_prefix,
            fn_ident,
            sanitize_capture_ident(&type_key)
        );

        registrations.push(quote! {
            #[doc(hidden)]
            #[allow(non_snake_case)]
            fn #capture_ident(
                world: &bevy::prelude::World,
            ) -> Option<bevy_extended_ui::lang::serde_json::Value> {
                world
                    .get_resource::<#resource_ty>()
                    .and_then(|value| bevy_extended_ui::lang::serde_json::to_value(value).ok())
            }

            bevy_extended_ui::lang::inventory::submit! {
                bevy_extended_ui::lang::HtmlSharedRegistration::Shared {
                    name: #name_lit,
                    path: concat!(module_path!(), "::", stringify!(#resource_ty)),
                    alias: #alias_lit,
                    auto_use: false,
                    capture: #capture_ident,
                }
            }
        });
    }

    quote! { #(#registrations)* }
}

fn extract_read_resource_type(ty: &Type) -> Option<Type> {
    let Type::Path(TypePath { path, .. }) = ty else {
        return None;
    };
    let segment = path.segments.last()?;
    let ident = segment.ident.to_string();

    if ident == "Option" {
        let PathArguments::AngleBracketed(args) = &segment.arguments else {
            return None;
        };
        let Some(GenericArgument::Type(inner)) = args.args.first() else {
            return None;
        };
        return extract_read_resource_type(inner);
    }

    if ident != "Res" && ident != "ResMut" {
        return None;
    }

    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    let Some(GenericArgument::Type(inner)) = args.args.first() else {
        return None;
    };

    Some(inner.clone())
}

fn simple_type_name_from_type(ty: &Type) -> Option<String> {
    let Type::Path(TypePath { path, .. }) = ty else {
        return None;
    };
    path.segments
        .last()
        .map(|segment| segment.ident.to_string())
}

/// Extracts the expected HTML event type from the first function argument.
fn extract_event_type(input_fn: &ItemFn) -> Result<Option<(syn::Ident, Type)>> {
    let Some(first_arg) = input_fn.sig.inputs.iter().next() else {
        return Ok(None);
    };

    let FnArg::Typed(pat_type) = first_arg else {
        return Ok(None);
    };

    let Type::Path(TypePath { path, .. }) = &*pat_type.ty else {
        return Ok(None);
    };

    let Some(last_segment) = path.segments.last() else {
        return Ok(None);
    };

    if last_segment.ident != "In" {
        return Ok(None);
    }

    let PathArguments::AngleBracketed(args) = &last_segment.arguments else {
        return Ok(None);
    };

    let Some(GenericArgument::Type(inner_type)) = args.args.first() else {
        return Ok(None);
    };

    let event_ident = match inner_type {
        Type::Path(TypePath { path, .. }) => path.segments.last().map(|seg| seg.ident.clone()),
        _ => None,
    };

    let Some(event_ident) = event_ident else {
        return Ok(None);
    };

    let variant = match event_ident.to_string().as_str() {
        "HtmlEvent" => format_ident!("HtmlEvent"),
        "HtmlClick" => format_ident!("HtmlClick"),
        "HtmlMouseDown" => format_ident!("HtmlMouseDown"),
        "HtmlMouseUp" => format_ident!("HtmlMouseUp"),
        "HtmlChange" => format_ident!("HtmlChange"),
        "HtmlSubmit" => format_ident!("HtmlSubmit"),
        "HtmlInit" => format_ident!("HtmlInit"),
        "HtmlMouseOut" => format_ident!("HtmlMouseOut"),
        "HtmlMouseOver" => format_ident!("HtmlMouseOver"),
        "HtmlFocus" => format_ident!("HtmlFocus"),
        "HtmlScroll" => format_ident!("HtmlScroll"),
        "HtmlWheel" => format_ident!("HtmlWheel"),
        "HtmlKeyDown" => format_ident!("HtmlKeyDown"),
        "HtmlKeyUp" => format_ident!("HtmlKeyUp"),
        "HtmlDragStart" => format_ident!("HtmlDragStart"),
        "HtmlDrag" => format_ident!("HtmlDrag"),
        "HtmlDragStop" => format_ident!("HtmlDragStop"),
        "HtmlTouchStart" => format_ident!("HtmlTouchStart"),
        "HtmlTouchMove" => format_ident!("HtmlTouchMove"),
        "HtmlTouchEnd" => format_ident!("HtmlTouchEnd"),
        _ => {
            return Err(syn::Error::new_spanned(
                &event_ident,
                "unsupported html event type; use HtmlEvent or a concrete Html* event",
            ));
        }
    };

    Ok(Some((variant, inner_type.clone())))
}

/// Marks a `*.component.rs` definition for the extended framework.
///
/// This attribute is intentionally a passthrough. Validation happens in
/// `bevy_extended_ui` by scanning component definition files.
#[proc_macro_attribute]
pub fn ui_component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Marks module entries that register `*.component.rs` files in a Rust build.
///
/// This marker is intentionally a passthrough and can be used for tooling that
/// injects/updates entries in `assets_components.rs` (or equivalent files).
#[proc_macro_attribute]
pub fn beu_registry(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Registers a typed item as shared template state (`@use "Type" as alias;`).
#[proc_macro_attribute]
pub fn html_shared(_attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_html_shared(item, false)
}

/// Like `html_shared`, but auto-imports the type with its default alias.
#[proc_macro_attribute]
pub fn html_use(_attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_html_shared(item, true)
}

fn expand_html_shared(item: TokenStream, auto_use: bool) -> TokenStream {
    let input = parse_macro_input!(item as Item);

    match input {
        Item::Struct(input) => expand_html_shared_nominal(input, auto_use).into(),
        Item::Enum(input) => expand_html_shared_enum(input, auto_use).into(),
        Item::Use(input) => match expand_html_shared_use(input, auto_use) {
            Ok(tokens) => tokens.into(),
            Err(err) => err.to_compile_error().into(),
        },
        other => {
            let err = syn::Error::new_spanned(
                &other,
                "#[html_shared]/#[html_use] support structs, enums, and single type imports",
            )
            .to_compile_error();
            TokenStream::from(quote! { #err #other })
        }
    }
}

fn expand_html_shared_nominal(input: ItemStruct, auto_use: bool) -> proc_macro2::TokenStream {
    if !input.generics.params.is_empty() {
        let err = syn::Error::new_spanned(
            &input.ident,
            "#[html_shared]/#[html_use] currently do not support generic structs",
        )
        .to_compile_error();
        return quote! { #err #input };
    }

    let ident = input.ident.clone();
    expand_html_shared_for_type(
        quote! { #input },
        quote! { #ident },
        ident.to_string(),
        quote! { concat!(module_path!(), "::", stringify!(#ident)) },
        auto_use,
        ident.span(),
    )
}

fn expand_html_shared_enum(input: ItemEnum, auto_use: bool) -> proc_macro2::TokenStream {
    if !input.generics.params.is_empty() {
        let err = syn::Error::new_spanned(
            &input.ident,
            "#[html_shared]/#[html_use] currently do not support generic enums",
        )
        .to_compile_error();
        return quote! { #err #input };
    }

    let ident = input.ident.clone();
    expand_html_shared_for_type(
        quote! { #input },
        quote! { #ident },
        ident.to_string(),
        quote! { concat!(module_path!(), "::", stringify!(#ident)) },
        auto_use,
        ident.span(),
    )
}

fn expand_html_shared_use(input: ItemUse, auto_use: bool) -> Result<proc_macro2::TokenStream> {
    let imported = extract_single_type_import(&input.tree)?;
    let local_ident = imported.local_ident;
    let type_name = imported.type_name;
    let path_lit = LitStr::new(&imported.target_path, local_ident.span());

    Ok(expand_html_shared_for_type(
        quote! { #input },
        quote! { #local_ident },
        type_name,
        quote! { #path_lit },
        auto_use,
        local_ident.span(),
    ))
}

fn expand_html_shared_for_type(
    item_tokens: proc_macro2::TokenStream,
    type_tokens: proc_macro2::TokenStream,
    shared_name: String,
    shared_path: proc_macro2::TokenStream,
    auto_use: bool,
    span: proc_macro2::Span,
) -> proc_macro2::TokenStream {
    let default_alias = to_default_alias(shared_name.as_str());

    let name_lit = LitStr::new(shared_name.as_str(), span);
    let alias_lit = LitStr::new(default_alias.as_str(), span);
    let capture_ident = format_ident!(
        "__html_shared_capture_{}",
        sanitize_capture_ident(shared_path.to_string().as_str())
    );

    quote! {
        #item_tokens

        impl bevy::prelude::Resource for #type_tokens {}

        #[doc(hidden)]
        fn #capture_ident(
            world: &bevy::prelude::World,
        ) -> Option<bevy_extended_ui::lang::serde_json::Value> {
            world
                .get_resource::<#type_tokens>()
                .and_then(|value| bevy_extended_ui::lang::serde_json::to_value(value).ok())
        }

        bevy_extended_ui::lang::inventory::submit! {
            bevy_extended_ui::lang::HtmlSharedRegistration::Shared {
                name: #name_lit,
                path: #shared_path,
                alias: #alias_lit,
                auto_use: #auto_use,
                capture: #capture_ident,
            }
        }
    }
}

struct ImportedType {
    target_path: String,
    local_ident: syn::Ident,
    type_name: String,
}

fn extract_single_type_import(tree: &UseTree) -> Result<ImportedType> {
    extract_single_type_import_inner(tree, String::new())
}

fn extract_single_type_import_inner(tree: &UseTree, prefix: String) -> Result<ImportedType> {
    match tree {
        UseTree::Path(path) => {
            let next = append_path_segment(prefix, path.ident.to_string());
            extract_single_type_import_inner(&path.tree, next)
        }
        UseTree::Name(name) => {
            let type_name = name.ident.to_string();
            Ok(ImportedType {
                target_path: append_path_segment(prefix, type_name.clone()),
                local_ident: name.ident.clone(),
                type_name,
            })
        }
        UseTree::Rename(rename) => {
            let type_name = rename.ident.to_string();
            Ok(ImportedType {
                target_path: append_path_segment(prefix, type_name.clone()),
                local_ident: rename.rename.clone(),
                type_name,
            })
        }
        other => Err(syn::Error::new_spanned(
            other,
            "#[html_shared]/#[html_use] on use items requires a single type import",
        )),
    }
}

fn append_path_segment(prefix: String, segment: String) -> String {
    if prefix.is_empty() {
        segment
    } else {
        format!("{prefix}::{segment}")
    }
}

fn to_default_alias(type_name: &str) -> String {
    let mut out = String::new();
    for (index, ch) in type_name.chars().enumerate() {
        if ch.is_uppercase() {
            if index != 0 {
                out.push('_');
            }
            for lower in ch.to_lowercase() {
                out.push(lower);
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn sanitize_capture_ident(raw: &str) -> String {
    let mut out = String::new();
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    while out.contains("__") {
        out = out.replace("__", "_");
    }
    out.trim_matches('_').to_string()
}
