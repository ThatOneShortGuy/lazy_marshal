use proc_macro::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, DataEnum, DataStruct, Fields};

#[proc_macro_derive(Marshal)]
pub fn marshal_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_marshal_macro(&ast)
}

fn marshal_struct(data_struct: &DataStruct) -> Option<proc_macro2::TokenStream> {
    data_struct
        .fields
        .clone()
        .into_iter()
        .map(|field| {
            let f = field.ident.unwrap();
            let ty = &field.ty;
            if let syn::Type::Reference(_) = ty {
                quote! {
                    self.#f.clone().marshal()
                }
            } else {
                quote! {
                    self.#f.marshal()
                }
            }
        })
        .reduce(|acc, n| {
            quote! {
                #acc.chain(#n)
            }
        })
}

fn marshal_enum(data_enum: &DataEnum) -> Option<proc_macro2::TokenStream> {
    let data = data_enum.variants.iter()
        .map(|var | {
            let args = match &var.fields {
                Fields::Named(a) => Some(syn::Error::new(a.span(), "Named fields are not supported").to_compile_error()),
                Fields::Unit => None,
                Fields::Unnamed(fields) => {
                    if fields.unnamed.len() > 1 {
                        Some(syn::Error::new(fields.unnamed.span(), "Only single unnamed fields are supported").to_compile_error())
                    } else {
                        let a = &fields.unnamed.first().unwrap().ty;
                        match a {
                            syn::Type::Verbatim(_) => panic!("Verbatim"),
                            _ => Some(quote! { })
                        }
                    }
                }
            };
            (&var.ident, args)
        })
        .enumerate()
        .map(|(i, (var_name, args))| {
                let i = i as u8;
                match args {
                    Some(args) => quote! { Self::#var_name(args) => {#args;MarshalIterator(Box::new(#i.marshal().chain(args.marshal())))} },
                    None =>  quote! {Self::#var_name =>  MarshalIterator(Box::new(#i.marshal()))},
                }
                
            }   
        )
        .reduce(|acc, val| quote! {
            #acc, #val
        })?;

    Some(quote! {
        match self {
            #data
        }
    })
}
fn unmarshal_enum(data_enum: &DataEnum) -> proc_macro2::TokenStream {
    let variants = data_enum.variants.iter()
        .map(|var| {
            let f = match &var.fields {
                Fields::Named(fields_named) => syn::Error::new(fields_named.span(), "Named fields are not supported").to_compile_error(),
                Fields::Unnamed(_fields_unnamed) => quote! {(UnMarshal::unmarshal(data)?)},
                Fields::Unit => quote! {},
            };
            (&var.ident, f)
        })
        .enumerate()
        .map(|(i, (id, field))| {
            let ii = i as u8;
            quote! { #ii => Self::#id #field}
        });

    quote! {
        let variant = u8::unmarshal(data)?;
        Ok(match variant {
            #(#variants, )*
            a => Err(MarshalError::InvalidData(format!(
                "Invalid enum varient: {a}"
            )))?,
        })
    }
}

fn impl_marshal_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let (impl_gen, ty_gen, where_gen) = &ast.generics.split_for_impl();

    let data = match &ast.data {
        syn::Data::Struct(data_struct) => marshal_struct(data_struct),
        syn::Data::Enum(data_enum) => marshal_enum(data_enum),
        // {
        //     return syn::Error::new(
        //         data_enum.enum_token.span(),
        //         "Marshalling enums with the derive macro isn't supported yet",
        //     )
        //     .into_compile_error()
        //     .into()
        // }
        syn::Data::Union(data_union) => {
            return syn::Error::new(
                data_union.union_token.span(),
                "Marshalling unions with the derive macro isn't supported yet",
            )
            .into_compile_error()
            .into()
        }
    };

    if let Some(d) = data {
        quote! {
            #[automatically_derived]
            impl #impl_gen Marshal for #name #ty_gen #where_gen {
                fn marshal(self) -> impl Iterator<Item = u8> {
                    #d
                }
            }
        }
        .into()
    } else {
        quote! {}.into()
    }
}

fn unmarshal_struct(data_struct: &DataStruct) -> proc_macro2::TokenStream {
    let fields =  data_struct.fields.iter().map(|field| {
        field.ident.as_ref().unwrap()
    });
    quote! {
        Ok(Self {
            #(#fields: UnMarshal::unmarshal(data)?),*
        })
    }
}

#[proc_macro_derive(UnMarshal)]
pub fn unmarshal_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_unmarshal_macro(&ast)
}

fn impl_unmarshal_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let (impl_gen, ty_gen, where_gen) = &ast.generics.split_for_impl();
    let unmarshal_body = match &ast.data {
        syn::Data::Struct(data_struct) => unmarshal_struct(data_struct),
        syn::Data::Enum(data_enum) => unmarshal_enum(data_enum),
        syn::Data::Union(data_union) => {
            syn::Error::new(
                data_union.union_token.span(),
                "Unmarshalling unions with the derive macro isn't supported",
            )
            .into_compile_error()
        }
    };

    let gen = quote! {
        #[automatically_derived]
        impl #impl_gen UnMarshal for #name #ty_gen #where_gen {
            fn unmarshal(data: &mut impl Iterator<Item = u8>) -> Result<Self, MarshalError> {
                #unmarshal_body
            }
        }
    };
    gen.into()
}
