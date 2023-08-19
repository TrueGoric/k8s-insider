use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, Data, DeriveInput, Field, Fields,
    Ident,
};

#[proc_macro_derive(TableOutputRow, attributes(name_column))]
pub fn derive_output_display(input: TokenStream) -> TokenStream {
    let parsed_input = parse_macro_input!(input as DeriveInput);
    let struct_ident = parsed_input.ident;
    let struct_generics_params = parsed_input.generics.params;
    let struct_generics_where = parsed_input.generics.where_clause;
    let parsed_struct = match parsed_input.data {
        Data::Struct(s) => s,
        _ => panic!("This derive macro is only applicable to named structs!"),
    };
    let fields = match parsed_struct.fields {
        Fields::Named(fields) => fields.named,
        _ => panic!("This derive macro is only applicable to named structs!"),
    };
    let name_field = get_name_column_field(&fields);
    let capitalized_field_names = get_capitalized_field_names(&fields);
    let field_names = fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect::<Vec<_>>();
    let field_count = field_names.len();

    let output = quote! {
        impl<#struct_generics_params> crate::output::TableOutputRow for #struct_ident<#struct_generics_params> #struct_generics_where {
            fn get_name(&self) -> ::std::string::String {
                ::std::string::ToString::to_string(&self.#name_field)
            }

            fn get_column_names() -> ::std::vec::Vec<::std::string::String> {
                ::std::vec![#(#capitalized_field_names.to_owned()),*]
            }

            fn get_column_count() -> usize {
                #field_count
            }

            fn get_row(&self) -> ::std::vec::Vec<::std::string::String> {
                ::std::vec![#(::std::string::ToString::to_string(&self.#field_names)),*]
            }
        }
    };

    output.into()
}

fn get_capitalized_field_names(
    fields: &Punctuated<Field, Comma>,
) -> impl Iterator<Item = String> + '_ {
    fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap().to_string().to_uppercase())
}

fn get_name_column_field(fields: &Punctuated<Field, Comma>) -> &Ident {
    fields
        .iter()
        .find(|f| {
            f.attrs
                .iter()
                .any(|a| a.meta.path().is_ident("name_column"))
        })
        .expect("This struct is missing a 'name_column' attribute!")
        .ident
        .as_ref()
        .unwrap()
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
