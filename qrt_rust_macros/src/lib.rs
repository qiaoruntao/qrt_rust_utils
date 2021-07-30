extern crate proc_macro;

use proc_macro::TokenStream;

use quote::quote;
use syn::{Data, DeriveInput, Ident};

#[proc_macro_derive(ToHashMap)]
pub fn derive_to_hashmap(input_struct: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input_struct as DeriveInput);

    // check for struct type and parse out fields
    let fields = match ast.data {
        Data::Struct(st) => st.fields,
        _ => panic!("Implementation must be a struct"),
    };

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
        .collect::<Vec<String>>();

    // get the name identifier of the struct input AST
    let name: &Ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    // start codegen for to_hashmap functionality that converts a struct into a hashmap
    let tokens = quote! {
        impl #impl_generics Into<HashMap<String, String>> for #name #ty_generics #where_clause {
            fn into(self) -> HashMap<String, String> {
                let mut map = HashMap::new();
                #(
                    map.insert(#keys.to_string(), self.#idents.to_string());
                )*
                map
            }
        }
    };
    TokenStream::from(tokens)
}
