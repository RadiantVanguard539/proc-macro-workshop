use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};
use quote::{quote, format_ident};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let _ = input;
    // Parse the macro input as a syn::DeriveInput syntax tree
    let ast = parse_macro_input!(input as DeriveInput);
    let ident = ast.ident.clone();

    // Have the macro produce a struct for the builder state, and a `builder`
    // function that creates an empty instance of the builder
    let builder_ident = format_ident!("{ident}Builder");
    let fields = match ast {
        syn::DeriveInput{
            data: syn::Data::Struct(
                syn::DataStruct{
                    fields: syn::Fields::Named (
                        syn::FieldsNamed{
                            named: fields,
                            ..
                        },
                    ),
                    ..
                },

            ),
            ..
        } => {
            fields
        },
        _ => unimplemented!("derive(Builder) only supports structs with named fields")
    };

    // Use DeriveInput fields to get builder struct fields
    let builder_fields = fields.iter().map(|field| {
        let field = field.clone();
        let id = field.ident.unwrap();
        let ty = field.ty;

        quote! {
            #id: std::option::Option<#ty>
        }
    });

    // Implement defaults for builder struct fields
    let builder_defaults = fields.iter().map(|field| {
        let field = field.clone();
        let id = field.ident.unwrap();
        
        quote! { #id: std::option::Option::None }
    });

    // Generate methods on the builder for setting a value of each of the struct
    // fields
    let setters = fields.iter().map(|field| {
        let field = field.clone();
        let id = field.ident.unwrap();
        let ty = field.ty;

        quote! {
            pub fn #id(&mut self, value: #ty) -> &mut Self {
                self.#id = std::option::Option::Some(value);
                self
            }
        }
    });

    // Check to ensure each build field has a value
    let build_checkers = fields.iter().map(|field| {
        let field = field.clone();
        let id = field.ident.unwrap();
        let err = format!("{id} was not set");

        quote! {
            if self.#id.is_none() {
                return std::result::Result::Err(#err.to_owned().into());
            }
        }
    });

    let build_fields = fields.iter().map(|field| {
        let field = field.clone();
        let id = field.ident.unwrap();

        quote! {
            #id: self.#id.clone().unwrap()
        }
    });

    // Output a builder struct
    let output = quote! {
        pub struct #builder_ident {
            #(#builder_fields),*
        }
        // Add setters to builder struct
        impl #builder_ident {
            #(#setters)*
            // Include the checkers when settings, using Box<dyn Error> for error message
            pub fn build(&mut self) -> std::result::Result<#ident, std::boxed::Box<dyn std::error::Error>> {
                #(#build_checkers);*

                std::result::Result::Ok(#ident {
                    #(#build_fields),*
                })
            }
        }
        impl #ident {
            pub fn builder() -> #builder_ident {
                #builder_ident { 
                    #(#builder_defaults),*
                }
            }   
        }
    };

    proc_macro::TokenStream::from(output)

    // unimplemented!()
}
