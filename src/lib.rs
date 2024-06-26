//! Oh, hi `mark`.
//! 
//! For those who crave more ergonomic marker traits.
//! 
//! ## Introduction
//! Marker traits are a common design pattern in Rust, used to denote certain properties or capabilities of types without requiring any additional implementation. However, managing marker traits can be tedious and often leads to boilerplate code. The `himark` crate aims to alleviate these issues by providing ergonomic utilities for working with marker traits.
//! 
//! ## About
//! The `himark` crate simplifies the usage of marker traits in Rust by offering two main features:
//! 
//! 1. **Automatic Implementation Generation**: Use `himark::mark` to automatically generate `impl` blocks for marker traits.
//! 2. **Trait Validation**: Use `himark::marker` to ensure that a trait meets the criteria for being a marker trait.
//! 
//! ## Usage
//! 
//! ### Generating Implementations for Marker Traits
//! The `himark::mark` attribute macro generates implementations for specified marker traits, reducing the need for boilerplate code.
//! 
//! #### Example:
//! ```rust
//! use himark::mark;
//! 
//! #[mark(MyMarkerTrait)]
//! struct MyStruct;
//! ```
//! 
//! This will automatically generate the following implementation:
//! ```rust
//! impl MyMarkerTrait for MyStruct {}
//! ```
//! 
//! ### Validating Marker Traits
//! The `himark::marker` attribute macro validates that a trait meets the criteria of being a marker trait, ensuring that it has no associated items and that all its super traits are also markers or auto traits.
//! 
//! #### Example:
//! ```rust
//! use himark::marker;
//! 
//! #[marker]
//! trait MyMarkerTrait {}
//! ```
//! 
//! This macro will produce a compile-time error if the trait does not meet the criteria for being a marker trait.
//! 
//! ### Recommended configuration
//! 
//! For best user experience we recommend importing `himark` as `hi` either with `use himark as hi;` or custom `Cargo.toml` configuration.
//! 
//! ```toml
//! [dependencies]
//! hi = { package = "himark", version = ... }
//! ```
//! 
//! And write code as allows:
//! 
//! ```rust
//! use himark as hi;
//! 
//! #[hi::mark(...)]
//! struct Foo { }
//! ```
//! 
//! ## Features
//! - **Automatic Implementation Generation**: Simplifies the process of implementing marker traits.
//! - **Trait Validation**: Ensures that your marker traits conform to the expected structure.
//! 
//! ## Contributing
//! Contributions are welcome! Please feel free to submit a pull request or open an issue on GitHub.
//! 
//! ## License
//! This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
 

extern crate proc_macro;

use proc_macro::{Punct, TokenStream};
use quote::{quote, quote_spanned};
use quote::TokenStreamExt;
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

struct Sealed {
    visibility: Option<syn::MetaList>,
}

// TODO: Parse Sealed path -- look into crate's source code.

impl quote::ToTokens for Sealed {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let path = quote! { ::sealed::sealed };
        
        let inner = self.visibility
            .as_ref()
            .map(|vis| quote! { pub(#vis) })
            .unwrap_or(proc_macro2::TokenStream::new());

        tokens.append_all(quote! { #[#path #inner] });
    }
}

#[derive(Default)]
struct Qualifiers {
    sealed: Option<Sealed>,
}

impl Qualifiers {
    pub fn sealed(&self) -> &Option<Sealed> {
        &self.sealed
    }
}

impl<'a> FromIterator<&'a syn::Attribute> for Qualifiers {
    fn from_iter<T: IntoIterator<Item = &'a syn::Attribute>>(iter: T) -> Self {
        let non_generic = iter.into_iter();

        fn inner_non_generic<'a>(attrs: impl Iterator<Item=&'a syn::Attribute>) -> Qualifiers {
            let mut default = Qualifiers::default();

            for attr in attrs {
                match &attr.meta {
                    syn::Meta::List(list) if list.path.is_ident("classifier") => {
                        list.parse_nested_meta(|classifier|{
                            if classifier.path.is_ident("sealed") {
                                default.is_sealed = true;
                            }
                            Ok(())
                        });
                    }
                    _ => continue,
                };
            }
            default
        }

        inner_non_generic(non_generic)
    }
}

#[proc_macro_attribute]
/// Attribute for marking types with marker traits.
pub fn mark(args: TokenStream, input: TokenStream) -> TokenStream {
    let parsed_input = parse_macro_input!(input as syn::DeriveInput);

    let qualifiers = Qualifiers::from_iter(&parsed_input.attrs);
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

pub fn markn(args: TokenStream, input: TokenStream) -> TokenStream {
    let parsed_input = parse_macro_input!(input as DeriveInput);

    let mut trait_names = Vec::new();

    // Parser to collect the trait names from the attribute arguments
    let parser = syn::meta::parser(|meta| match meta {
        Meta::Path(path) => {
            trait_names.push(path);
            Ok(())
        }
        _ => Err(meta.error("expected trait name")),
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

    // Check for sealed attribute
    let is_sealed = parsed_input.attrs.iter().any(is_sealed_attribute);

    if is_sealed {
        gen.extend(quote_spanned! {span=>
            impl #impl_generics sealed::Sealed for #ident #ty_generics #where_clause {}
        });
    }

    gen.into()
}
