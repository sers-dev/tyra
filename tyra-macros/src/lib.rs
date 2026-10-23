extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Attribute, Expr, Token};
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};

enum TyraDefault {
    None,
    Some(Expr),
    Empty,
}

struct TyraAttribute {
    default: TyraDefault,
}

impl Parse for TyraAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut default = TyraDefault::None;
        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            if ident == "default" {
                if input.peek(Token![=]) {
                    input.parse::<Token![=]>()?;
                    let expr: Expr = input.parse()?;
                    // Check if it's a string literal and try to parse it as an expression
                    if let Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(lit_str), .. }) = &expr {
                        if let Ok(parsed_expr) = lit_str.parse() {
                            default = TyraDefault::Some(parsed_expr);
                        } else {
                            default = TyraDefault::Some(expr);
                        }
                    } else {
                        default = TyraDefault::Some(expr);
                    }
                } else if input.peek(syn::token::Paren) {
                    let content;
                    syn::parenthesized!(content in input);
                    let expr: Expr = content.parse()?;
                    // Also check for string literal in parenthesized case
                    if let Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(lit_str), .. }) = &expr {
                        if let Ok(parsed_expr) = lit_str.parse() {
                            default = TyraDefault::Some(parsed_expr);
                        } else {
                            default = TyraDefault::Some(expr);
                        }
                    } else {
                        default = TyraDefault::Some(expr);
                    }
                } else {
                    default = TyraDefault::Empty;
                }
            } else {
                // Skip other idents if any
                if input.peek(Token![=]) {
                    input.parse::<Token![=]>()?;
                    input.parse::<Expr>()?;
                }
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(TyraAttribute { default })
    }
}

fn get_tyra_default(attrs: &[Attribute]) -> TyraDefault {
    for attr in attrs {
        if attr.path.is_ident("tyra") {
            let res = attr.parse_args::<TyraAttribute>();
            if let Ok(tyra_attr) = res {
                if let TyraDefault::None = tyra_attr.default {
                    continue;
                }
                return tyra_attr.default;
            }
        }
    }
    TyraDefault::None
}

fn is_option(ty: &syn::Type) -> bool {
    if let syn::Type::Path(tp) = ty {
        if tp.qself.is_none() {
            if let Some(last_segment) = tp.path.segments.last() {
                return last_segment.ident == "Option";
            }
        }
    }
    false
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

                    let tyra_default = get_tyra_default(&field.attrs);
                    let default_expr = match tyra_default {
                        TyraDefault::Some(expr) => Some(expr),
                        TyraDefault::Empty => {
                            if is_option(field_type) {
                                Some(syn::parse_quote! { None })
                            } else {
                                panic!("Empty default is only allowed for Option fields");
                            }
                        }
                        TyraDefault::None => None,
                    };

                    if let Some(default_expr) = default_expr {
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

                    let tyra_default = get_tyra_default(&field.attrs);
                    let default_expr = match tyra_default {
                        TyraDefault::Some(expr) => Some(expr),
                        TyraDefault::Empty => {
                            if is_option(field_type) {
                                Some(syn::parse_quote! { None })
                            } else {
                                panic!("Empty default is only allowed for Option fields");
                            }
                        }
                        TyraDefault::None => None,
                    };

                    if let Some(default_expr) = default_expr {
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

                    let tyra_default = get_tyra_default(&field.attrs);
                    let default_expr = match tyra_default {
                        TyraDefault::Some(expr) => Some(expr),
                        TyraDefault::Empty => {
                            if is_option(field_type) {
                                Some(syn::parse_quote! { None })
                            } else {
                                panic!("Empty default is only allowed for Option fields");
                            }
                        }
                        TyraDefault::None => None,
                    };

                    if let Some(default_expr) = default_expr {
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
    let mut factory_struct_fields = Vec::new();
    let mut actor_field_inits = Vec::new();

    if let syn::Data::Struct(data) = &input.data {
        match &data.fields {
            syn::Fields::Named(fields) => {
                for field in &fields.named {
                    let field_name = &field.ident;
                    let field_type = &field.ty;

                    let tyra_default = get_tyra_default(&field.attrs);
                    let default_expr = match tyra_default {
                        TyraDefault::Some(expr) => Some(expr),
                        TyraDefault::Empty => {
                            if is_option(field_type) {
                                Some(syn::parse_quote! { None })
                            } else {
                                panic!("Empty default is only allowed for Option fields");
                            }
                        }
                        TyraDefault::None => None,
                    };

                    if let Some(default_expr) = default_expr {
                        factory_field_inits.push(quote! { #field_name: #default_expr });
                        factory_struct_fields.push(quote! { pub #field_name: #field_type });
                        actor_field_inits.push(quote! { #field_name: self.#field_name.clone() });
                    } else {
                        let type_str = field_type.to_token_stream().to_string();
                        let is_ctx = type_str.contains("ActorContext") && type_str.contains("Self");

                        if is_ctx {
                            actor_field_inits.push(quote! { #field_name: context });
                        } else {
                            factory_params.push(quote! { #field_name: #field_type });
                            factory_field_inits.push(quote! { #field_name });
                            factory_struct_fields.push(quote! { pub #field_name: #field_type });
                            actor_field_inits.push(quote! { #field_name: self.#field_name.clone() });
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let factory_name = quote::format_ident!("{}Factory", name);

    let expanded = quote! {
        #[derive(Clone)]
        pub struct #factory_name #ty_generics #where_clause {
            #( #factory_struct_fields ),*
        }
        impl #impl_generics #name #ty_generics #where_clause {
            pub fn factory(#( #factory_params ),*) -> #factory_name #ty_generics {
                #factory_name {
                    #( #factory_field_inits ),*
                }
            }
        }
        impl #impl_generics tyra::prelude::ActorFactory<#name #ty_generics> for #factory_name #ty_generics #where_clause {
            fn new_actor(&mut self, context: tyra::prelude::ActorContext<#name #ty_generics>) -> Result<#name #ty_generics, Box<dyn std::error::Error>> {
                Ok(#name {
                    #( #actor_field_inits ),*
                })
            }
        }
    };

    TokenStream::from(expanded)
}
