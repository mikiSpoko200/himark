extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, ItemTrait};

#[proc_macro_attribute]
/// Attribute for use on traits which verifies that they are indeed markers.
///
/// Trait is considered a marker if:
/// - has no associated items
/// - all its super traits are also markers or auto traits
pub fn marker(_: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input_trait = parse_macro_input!(input as ItemTrait);

    if !input_trait.items.is_empty() {
        return syn::Error::new_spanned(
            input_trait,
            "The #[marker] attribute can only be applied to empty traits",
        )
        .to_compile_error()
        .into();
    }

    let gen = quote! {
        #input_trait
    };

    gen.into()
}

#[proc_macro_attribute]
/// Attribute for marking types with marker traits.
pub fn mark(args: TokenStream, input: TokenStream) -> TokenStream {
    let parsed_input = parse_macro_input!(input as syn::DeriveInput);

    let mut trait_names = Vec::new();

    // Parser to collect the trait names from the attribute arguments
    let parser = syn::meta::parser(|meta| match meta.value() {
        Err(_) => {
            trait_names.push(meta.path);
            Ok(())
        }
        Ok(_) => Err(meta.error("expected trait name")),
    });

    parse_macro_input!(args with parser);

    let ident = &parsed_input.ident;
    let span = parsed_input.ident.span();
    let generics = &parsed_input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let impls = trait_names.iter().map(|trait_name| {
        quote_spanned! {span=>
            impl #impl_generics #trait_name for #ident #ty_generics #where_clause {}
        }
    });

    let mut gen = quote::quote! { #parsed_input };
    gen.extend(impls);

    gen.into()
}
