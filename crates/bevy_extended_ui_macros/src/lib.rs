use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    FnArg, GenericArgument, ItemFn, LitStr, PathArguments, Result, Type, TypePath,
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
        "HtmlChange" => format_ident!("HtmlChange"),
        "HtmlSubmit" => format_ident!("HtmlSubmit"),
        "HtmlInit" => format_ident!("HtmlInit"),
        "HtmlMouseOut" => format_ident!("HtmlMouseOut"),
        "HtmlMouseOver" => format_ident!("HtmlMouseOver"),
        "HtmlFocus" => format_ident!("HtmlFocus"),
        "HtmlScroll" => format_ident!("HtmlScroll"),
        "HtmlKeyDown" => format_ident!("HtmlKeyDown"),
        "HtmlKeyUp" => format_ident!("HtmlKeyUp"),
        "HtmlDragStart" => format_ident!("HtmlDragStart"),
        "HtmlDrag" => format_ident!("HtmlDrag"),
        "HtmlDragStop" => format_ident!("HtmlDragStop"),
        _ => {
            return Err(syn::Error::new_spanned(
                &event_ident,
                "unsupported html event type; use HtmlEvent or a concrete Html* event",
            ));
        }
    };

    Ok(Some((variant, inner_type.clone())))
}
