mod parse_enum;
mod templates;

extern crate proc_macro;

use proc_macro::TokenStream;
use std::collections::HashMap;
use syn::DeriveInput;
use tera::{Context, Tera};

use crate::parse_enum::{
    create_marker_enums, fetch_fields_from_enum, fetch_impl_generics,
    fetch_name_with_generic_params, get_marker,
};
use crate::templates::{FROM_TEMPLATE, GET_VARIANT_TEMPLATE, TRY_FROM_TEMPLATE};

#[proc_macro_derive(EnumConversions)]
pub fn enum_conversions_derive(input: TokenStream) -> TokenStream {
    let enum_ast = syn::parse(input).unwrap();
    impl_conversions(&enum_ast)
}

/// Implement the helper trait `GetVariant`.
fn impl_get_variant(
    name: &str,
    fullname: &str,
    where_clause: &str,
    impl_generics: &str,
    field_map: &HashMap<String, String>,
    templater: &Tera,
) -> TokenStream {
    let mut impl_string = String::new();
    for (field, ty) in field_map.iter() {
        let mut context = Context::new();
        context.insert("generics", impl_generics);
        context.insert("Type", ty);
        context.insert("Marker", &get_marker(name, field));
        context.insert("fullname", fullname);
        context.insert("name", name);
        context.insert("field", field);
        context.insert("Where", where_clause);
        impl_string.push_str(
            &templater
                .render("get_variant", &context)
                .expect("Failed to render the GetVariant template"),
        );
    }
    impl_string.parse().unwrap()
}

/// Implement the `TryFrom` traits for each type in the
/// enum. Uses the `GetVariant` helper trait and marker structs
/// to avoid generic parameter ambiguity and restrictions
/// to `'static` lifetimes.
fn impl_try_from(
    name: &str,
    fullname: &str,
    where_clause: &str,
    impl_generics: &str,
    field_map: &HashMap<String, String>,
    templater: &Tera,
) -> TokenStream {
    let mut impl_string = String::new();
    for (field, ty) in field_map.iter() {
        let mut where_string = where_clause.to_string();
        let marker_bound = if where_clause.is_empty() {
            format!(
                "where\n {}: GetVariant<{}, {}>",
                fullname,
                ty,
                get_marker(name, field)
            )
        } else {
            format!(
                "\n {}: GetVariant<{}, {}>",
                fullname,
                ty,
                get_marker(name, field)
            )
        };
        where_string.push_str(&marker_bound);
        let mut context = Context::new();
        context.insert("generics", impl_generics);
        context.insert("Type", ty);
        context.insert("fullname", fullname);
        context.insert("name", name);
        context.insert("Where", &where_string);
        impl_string.push_str(
            &templater
                .render("try_from", &context)
                .expect("Failed to render the TryFrom template"),
        );
    }
    impl_string.parse().unwrap()
}

fn impl_from(
    fullname: &str,
    where_clause: &str,
    impl_generics: &str,
    field_map: &HashMap<String, String>,
    templater: &Tera,
) -> TokenStream {
    let mut impl_string = String::new();
    for (field, ty) in field_map.iter() {
        let mut context = Context::new();
        context.insert("generics", impl_generics);
        context.insert("Type", ty);
        context.insert("fullname", fullname);
        context.insert("field", field);
        context.insert("Where", where_clause);
        impl_string.push_str(
            &templater
                .render("from", &context)
                .expect("Failed to render the From template"),
        );
    }
    impl_string.parse().unwrap()
}

/// Implements ContainsVariant, GetVariant, SetVariant, and CreateVariantFrom traits
fn impl_conversions(ast: &DeriveInput) -> TokenStream {
    let mut tera = Tera::new("/dev/null/*").unwrap();
    tera.add_raw_template("get_variant", GET_VARIANT_TEMPLATE)
        .unwrap();
    tera.add_raw_template("try_from", TRY_FROM_TEMPLATE)
        .unwrap();
    tera.add_raw_template("from", FROM_TEMPLATE).unwrap();
    let mut tokens: TokenStream = "".parse().unwrap();

    let name = &ast.ident.to_string();
    let fullname = fetch_name_with_generic_params(ast);
    let (impl_generics, where_clause) = fetch_impl_generics(ast);
    let field_map = fetch_fields_from_enum(ast);

    tokens.extend::<TokenStream>(create_marker_enums(name, &field_map));
    tokens.extend::<TokenStream>(impl_get_variant(
        name,
        &fullname,
        &where_clause,
        &impl_generics,
        &field_map,
        &tera,
    ));
    tokens.extend::<TokenStream>(impl_try_from(
        name,
        &fullname,
        &where_clause,
        &impl_generics,
        &field_map,
        &tera,
    ));
    tokens.extend::<TokenStream>(impl_from(
        &fullname,
        &where_clause,
        &impl_generics,
        &field_map,
        &tera,
    ));
    tokens
}
