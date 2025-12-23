use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    token::Eq,
    ItemFn,
    LitStr,
    Result,
};

struct HtmlFnAttr {
    name: LitStr,
}

impl Parse for HtmlFnAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Eq) {
            let _eq: Eq = input.parse()?;
            Ok(Self { name: input.parse()? })
        } else {
            Ok(Self { name: input.parse()? })
        }
    }
}

#[proc_macro_attribute]
pub fn html_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    let HtmlFnAttr { name } = parse_macro_input!(attr as HtmlFnAttr);
    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_ident = input_fn.sig.ident.clone();
    let builder_ident = format_ident!("__html_fn_build_{}", fn_ident);

    let expanded = quote! {
        #input_fn

        #[doc(hidden)]
        fn #builder_ident(world: &mut bevy::prelude::World) -> bevy::ecs::system::SystemId<(), ()> {
            world.register_system(#fn_ident)
        }

        bevy_extended_ui::html::inventory::submit! {
            bevy_extended_ui::html::HtmlFnRegistration {
                name: #name,
                build: #builder_ident,
            }
        }
    };

    expanded.into()
}