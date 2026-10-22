extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Actor)]
pub fn actor_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics tyra::prelude::Actor for #name #ty_generics #where_clause {}
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(ActorMessage)]
pub fn actor_message_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut new_params = Vec::new();
    let mut new_body = quote! {};

    if let syn::Data::Struct(data) = &input.data {
        match &data.fields {
            syn::Fields::Named(fields) => {
                let mut field_names = Vec::new();
                for field in &fields.named {
                    let field_name = &field.ident;
                    let field_type = &field.ty;
                    new_params.push(quote! { #field_name: #field_type });
                    field_names.push(field_name);
                }
                new_body = quote! {
                    Self {
                        #( #field_names ),*
                    }
                };
            }
            syn::Fields::Unnamed(fields) => {
                let mut field_names = Vec::new();
                for (i, field) in fields.unnamed.iter().enumerate() {
                    let field_name = quote::format_ident!("arg{}", i);
                    let field_type = &field.ty;
                    new_params.push(quote! { #field_name: #field_type });
                    field_names.push(field_name);
                }
                new_body = quote! {
                    Self (
                        #( #field_names ),*
                    )
                };
            }
            syn::Fields::Unit => {
                new_body = quote! { Self };
            }
        }
    }

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            pub fn new(#( #new_params ),*) -> Self {
                #new_body
            }
        }
        impl #impl_generics tyra::prelude::ActorMessage for #name #ty_generics #where_clause {}
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(ActorFactory)]
pub fn actor_factory_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics tyra::prelude::ActorFactory<#name #ty_generics> for #name #ty_generics #where_clause {
            fn new_actor(&mut self, _context: tyra::prelude::ActorContext<Self>) -> Result<Self, Box<dyn std::error::Error>> {
                Ok(self.clone())
            }
        }
    };

    TokenStream::from(expanded)
}
