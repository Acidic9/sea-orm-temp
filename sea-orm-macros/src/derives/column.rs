use heck::{MixedCase, SnakeCase};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{Data, DataEnum, Fields, Variant};

pub fn impl_default_as_str(ident: &Ident, data: &Data) -> syn::Result<TokenStream> {
    let variants = match data {
        syn::Data::Enum(DataEnum { variants, .. }) => variants,
        _ => {
            return Ok(quote_spanned! {
                ident.span() => compile_error!("you can only derive DeriveColumn on enums");
            })
        }
    };

    let variant: Vec<TokenStream> = variants
        .iter()
        .map(|Variant { ident, fields, .. }| match fields {
            Fields::Named(_) => quote! { #ident{..} },
            Fields::Unnamed(_) => quote! { #ident(..) },
            Fields::Unit => quote! { #ident },
        })
        .collect();

    let name: Vec<TokenStream> = variants
        .iter()
        .map(|v| {
            let ident = v.ident.to_string().to_snake_case();
            quote! { #ident }
        })
        .collect();

    Ok(quote!(
        impl #ident {
            fn default_as_str(&self) -> &str {
                match self {
                    #(Self::#variant => #name),*
                }
            }
        }
    ))
}

pub fn expand_derive_column(ident: &Ident, data: &Data) -> syn::Result<TokenStream> {
    let impl_iden = expand_derive_custom_column(ident, data)?;
    let parse_error_iden = format_ident!("Parse{}Err", ident);

    let data_enum = match data {
        Data::Enum(data_enum) => data_enum,
        _ => panic!("DeriveColumn can only be used on an enum"),
    };

    let columns = data_enum.variants.iter().map(|column| {
        let column_iden = column.ident.clone();
        let column_str_snake = column_iden.to_string().to_snake_case();
        let column_str_mixed = column_iden.to_string().to_mixed_case();
        quote!(
            #column_str_snake | #column_str_mixed => Ok(#ident::#column_iden)
        )
    });

    Ok(quote!(
        #impl_iden

        impl #ident {
            pub fn from_graphql_ctx(ctx: &async_graphql::context::Context<'_>) -> Vec<Self> {
                ctx.field()
                    .selection_set()
                    .filter_map(|selection| std::str::FromStr::from_str(selection.name()).ok())
                    .collect()
            }
        }

        #[derive(Debug, Clone, Copy)]
        pub struct #parse_error_iden;

        impl std::str::FromStr for #ident {
            type Err = #parse_error_iden;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    #(#columns),*,
                    _ => Err(#parse_error_iden),
                }
            }
        }

        impl sea_orm::IdenStatic for #ident {
            fn as_str(&self) -> &str {
                self.default_as_str()
            }
        }
    ))
}

pub fn expand_derive_custom_column(ident: &Ident, data: &Data) -> syn::Result<TokenStream> {
    let impl_default_as_str = impl_default_as_str(ident, data)?;

    Ok(quote!(
        #impl_default_as_str

        impl sea_orm::Iden for #ident {
            fn unquoted(&self, s: &mut dyn std::fmt::Write) {
                write!(s, "{}", self.as_str()).unwrap();
            }
        }
    ))
}
