use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    FnArg, GenericArgument, ItemFn, ItemStruct, LitStr, PathArguments, Result, Type, TypePath,
    parse::{Parse, ParseStream},
    parse_macro_input,
    token::Eq,
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

/// Registers a typed struct as shared template state (`@use "Type" as alias;`).
#[proc_macro_attribute]
pub fn html_shared(_attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_html_shared(item, false)
}

/// Like `html_shared`, but auto-imports the struct with its default alias.
#[proc_macro_attribute]
pub fn html_use(_attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_html_shared(item, true)
}

fn expand_html_shared(item: TokenStream, auto_use: bool) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    if !input.generics.params.is_empty() {
        let err = syn::Error::new_spanned(
            &input.ident,
            "#[html_shared]/#[html_use] currently do not support generic structs",
        )
        .to_compile_error();
        return TokenStream::from(quote! { #err #input });
    }

    let struct_ident = input.ident.clone();
    let shared_name = struct_ident.to_string();
    let default_alias = to_default_alias(shared_name.as_str());

    let name_lit = LitStr::new(shared_name.as_str(), struct_ident.span());
    let alias_lit = LitStr::new(default_alias.as_str(), struct_ident.span());
    let capture_ident = format_ident!("__html_shared_capture_{}", default_alias);

    let expanded = quote! {
        #input

        impl bevy::prelude::Resource for #struct_ident {}

        #[doc(hidden)]
        fn #capture_ident(
            world: &bevy::prelude::World,
        ) -> Option<bevy_extended_ui::lang::serde_json::Value> {
            world
                .get_resource::<#struct_ident>()
                .and_then(|value| bevy_extended_ui::lang::serde_json::to_value(value).ok())
        }

        bevy_extended_ui::lang::inventory::submit! {
            bevy_extended_ui::lang::HtmlSharedRegistration::Shared {
                name: #name_lit,
                alias: #alias_lit,
                auto_use: #auto_use,
                capture: #capture_ident,
            }
        }
    };

    expanded.into()
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
