use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, ExprLit, Fields, Meta, Type};


///
/// #[repr(u32)]
/// #[derive(Debug, NumToEnum)]
/// pub enum Cpu {
///     X84 = 1,
///     Arm = 2,
///     Hello = 3 | 4,
///     Unknown(u32),
/// }
///  let v: u32 = Cpu::Arm.into();
///  let cpu: Cpu = (3|4).into();
#[proc_macro_derive(NumToEnum)]
pub fn num_to_enum(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let enum_name = &ast.ident;
    if let Data::Enum(data_enum) = &ast.data {
        let enum_base_type = ast
            .attrs
            .iter()
            .find_map(|attr| {
                if attr.path().is_ident("repr") {
                    if let Meta::List(meta_list) = &attr.meta {
                        return Some(meta_list.tokens.clone());
                    }
                }
                Some(quote! {u32})
            })
            .unwrap();
        let mut into_fields = vec![];
        let mut from_fields = vec![];
        for variant in &data_enum.variants {
            let field_name = &variant.ident;
            match &variant.fields {
                Fields::Named(_) => {}
                Fields::Unnamed(v) => {
                    if let Some(name) = v.unnamed.first() {
                        if let Type::Path(ty, ..) = &name.ty {
                            if let Some(_seg) = ty.path.segments.first() {
                                into_fields.push(quote! {
                                  #enum_name::#field_name(value) => value,
                                });
                                from_fields.push(quote! {
                                   value => #enum_name::#field_name(value),
                                });
                            }
                        }
                    }
                }
                Fields::Unit => {
                    if let Some((_, value)) = &variant.discriminant {
                        if let syn::Expr::Lit(ExprLit { lit, .. }) = value {
                            into_fields.push(quote! {
                               #enum_name::#field_name => #lit,
                            });
                            from_fields.push(quote! {
                               #lit => #enum_name::#field_name,
                            });
                        } else if let syn::Expr::Binary(ex) = value {
                            into_fields.push(quote! {
                               #enum_name::#field_name => #ex,
                            });
                            from_fields.push(quote! {
                               value if value == #ex => #enum_name::#field_name,
                            });
                        }
                    }
                }
            }
        }
        let expanded = quote! {
            impl Into<#enum_base_type> for #enum_name {
                fn into(self) -> #enum_base_type {
                    match self {
                        #(#into_fields)*
                    }
                }
            }
            impl From<#enum_base_type> for #enum_name{
                fn from(value: #enum_base_type) -> Self {
                    match value {
                        #(#from_fields)*
                        _ => panic!("can not match {}",value)
                    }
                }
            }
        };
        TokenStream::from(expanded)
    } else {
        syn::Error::new_spanned(ast, "NumToEnum Only added in Enum!")
            .to_compile_error()
            .into()
    }
}
