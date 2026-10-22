extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Attribute, Expr, Meta, NestedMeta, Lit};
use quote::ToTokens;

fn get_tyra_default(attrs: &[Attribute]) -> Option<Expr> {
    for attr in attrs {
        if let Ok(meta) = attr.parse_meta() {
            if meta.path().is_ident("tyra") {
                if let Meta::List(list) = meta {
                    for nested in list.nested {
                        if let NestedMeta::Meta(Meta::NameValue(nv)) = nested {
                            if nv.path.is_ident("default") {
                                if let Lit::Str(lit_str) = &nv.lit {
                                    return lit_str.parse().ok();
                                }
                                return Some(syn::parse2(nv.lit.to_token_stream()).ok()?);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

#[proc_macro_derive(Actor, attributes(tyra))]
pub fn actor_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut new_params = Vec::new();
    let mut field_inits = Vec::new();

    if let syn::Data::Struct(data) = &input.data {
        match &data.fields {
            syn::Fields::Named(fields) => {
                let mut ctx_param = None;
                for field in &fields.named {
                    let field_name = &field.ident;
                    let field_type = &field.ty;

                    if let Some(default_expr) = get_tyra_default(&field.attrs) {
                        field_inits.push(quote! { #field_name: #default_expr });
                    } else {
                        let is_ctx = field_name.as_ref().map(|n| n == "ctx" || n == "context").unwrap_or(false);
                        if is_ctx {
                             ctx_param = Some(quote! { context: #field_type });
                             field_inits.push(quote! { #field_name: context });
                        } else {
                            new_params.push(quote! { #field_name: #field_type });
                            field_inits.push(quote! { #field_name });
                        }
                    }
                }
                if let Some(ctx) = ctx_param {
                    new_params.push(ctx);
                }
            }
            _ => {}
        }
    }

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            pub fn new(#( #new_params ),*) -> Self {
                Self {
                    #( #field_inits ),*
                }
            }
        }
        impl #impl_generics tyra::prelude::Actor for #name #ty_generics #where_clause {}
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(ActorMessage, attributes(tyra))]
pub fn actor_message_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut new_params = Vec::new();
    let mut field_inits = Vec::new();
    let mut is_named = false;
    let mut is_tuple = false;

    if let syn::Data::Struct(data) = &input.data {
        match &data.fields {
            syn::Fields::Named(fields) => {
                is_named = true;
                for field in &fields.named {
                    let field_name = &field.ident;
                    let field_type = &field.ty;
                    if let Some(default_expr) = get_tyra_default(&field.attrs) {
                        field_inits.push(quote! { #field_name: #default_expr });
                    } else {
                        new_params.push(quote! { #field_name: #field_type });
                        field_inits.push(quote! { #field_name });
                    }
                }
            }
            syn::Fields::Unnamed(fields) => {
                is_tuple = true;
                for (i, field) in fields.unnamed.iter().enumerate() {
                    let field_type = &field.ty;
                    if let Some(default_expr) = get_tyra_default(&field.attrs) {
                        field_inits.push(quote! { #default_expr });
                    } else {
                        let arg_name = quote::format_ident!("arg{}", i);
                        new_params.push(quote! { #arg_name: #field_type });
                        field_inits.push(quote! { #arg_name });
                    }
                }
            }
            syn::Fields::Unit => {}
        }
    }

    let new_body = if is_named {
        quote! {
            Self {
                #( #field_inits ),*
            }
        }
    } else if is_tuple {
        quote! {
            Self (
                #( #field_inits ),*
            )
        }
    } else {
        quote! { Self }
    };

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

#[proc_macro_derive(ActorFactory, attributes(tyra))]
pub fn actor_factory_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut factory_params = Vec::new();
    let mut factory_field_inits = Vec::new();
    let mut actor_field_inits = Vec::new();

    if let syn::Data::Struct(data) = &input.data {
        match &data.fields {
            syn::Fields::Named(fields) => {
                for field in &fields.named {
                    let field_name = &field.ident;
                    let field_type = &field.ty;

                    if let Some(default_expr) = get_tyra_default(&field.attrs) {
                        factory_field_inits.push(quote! { #field_name: #default_expr });
                        actor_field_inits.push(quote! { #field_name: self.#field_name.clone() });
                    } else {
                        let type_str = field_type.to_token_stream().to_string();
                        let is_ctx = type_str.contains("ActorContext") && type_str.contains("Self");

                        if is_ctx {
                            factory_field_inits.push(quote! { #field_name: tyra::prelude::ActorContext::dummy() });
                            actor_field_inits.push(quote! { #field_name: context });
                        } else {
                            factory_params.push(quote! { #field_name: #field_type });
                            factory_field_inits.push(quote! { #field_name });
                            actor_field_inits.push(quote! { #field_name: self.#field_name.clone() });
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            pub fn factory(#( #factory_params ),*) -> Self {
                Self {
                    #( #factory_field_inits ),*
                }
            }
        }
        impl #impl_generics tyra::prelude::ActorFactory<#name #ty_generics> for #name #ty_generics #where_clause {
            fn new_actor(&mut self, context: tyra::prelude::ActorContext<Self>) -> Result<Self, Box<dyn std::error::Error>> {
                Ok(Self {
                    #( #actor_field_inits ),*
                })
            }
        }
    };

    TokenStream::from(expanded)
}
