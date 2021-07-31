extern crate proc_macro;

use proc_macro::TokenStream;

use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, NestedMeta};

#[proc_macro_derive(ToHashMap, attributes(serde))]
pub fn derive_to_hashmap(input_struct: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input_struct as DeriveInput);

    // check for struct type and parse out fields
    let fields = match ast.data {
        Data::Struct(st) => st.fields,
        _ => panic!("Implementation must be a struct"),
    };
    let mut rename_map = std::collections::HashMap::new();
    match fields {
        Fields::Named(_) => {
            for field in fields.iter() {
                for attr in field.attrs.iter() {
                    let field_name = field.ident.as_ref().unwrap().to_string();
                    let is_already_defined = rename_map.contains_key(&field_name);
                    if is_already_defined {
                        panic!("Field name {} is already defined", &field_name);
                    }
                    match attr.parse_meta() {
                        Ok(syn::Meta::List(lst)) => {
                            for sub_attr in lst.nested.iter() {
                                match sub_attr {
                                    NestedMeta::Meta(syn::Meta::NameValue(name_value)) => {
                                        let sub_attr_name = name_value.path.get_ident().unwrap().to_string();
                                        if sub_attr_name != "rename" {
                                            break;
                                        }
                                        let alternative_name = match &name_value.lit {
                                            syn::Lit::Str(value) => value.value(),
                                            _ => { break; }
                                        };
                                        rename_map.insert(field_name, alternative_name);
                                        // there should only be one rename, ignore other renames
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        _ => {
            panic!("All fields must be named in struct");
        }
    }


    // parse out all the field names in the struct as `Ident`s
    let idents: Vec<&Ident> = fields
        .iter()
        .filter_map(|field| field.ident.as_ref())
        .collect::<Vec<&Ident>>();

    // convert all the field names into strings
    let keys: Vec<String> = idents
        .clone()
        .iter()
        .map(|ident| ident.to_string())
        .map(|ident| rename_map.get(&ident).unwrap_or(&ident).clone())
        .collect::<Vec<String>>();

    // get the name identifier of the struct input AST
    let name: &Ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    // start codegen for to_hashmap functionality that converts a struct into a hashmap
    let tokens = quote! {
        impl #impl_generics Into<std::collections::HashMap<String, String>> for #name #ty_generics #where_clause {
            fn into(self) -> std::collections::HashMap<String, String> {
                let mut map = std::collections::HashMap::new();
                #(
                    map.insert(#keys.to_string(), self.#idents.to_string());
                )*
                map
            }
        }
    };
    TokenStream::from(tokens)
}
