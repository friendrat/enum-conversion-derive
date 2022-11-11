use super::*;
use quote::ToTokens;
use std::collections::HashMap;
use syn::Data;

/// This functions determines the name of the enum with generic
/// params attached.
///
/// # Example
/// ```
/// enum Enum<'a, T: 'a + Debug, const X: usize> {
///     F1(T),
///     F2(X)
/// }
/// ```
/// This function should return `Enum<'a, T, X>`
pub fn fetch_name_with_generic_params(ast: &DeriveInput) -> String {
    let mut param_string = String::new();
    for param in ast.generics.params.iter() {
        let next = match param {
            syn::GenericParam::Type(type_) => type_.ident.to_token_stream(),
            syn::GenericParam::Lifetime(life_def) => life_def.lifetime.to_token_stream(),
            syn::GenericParam::Const(constant) => constant.ident.to_token_stream(),
        };
        param_string.push_str(&format!("{},", next));
    }
    param_string.pop();
    if !param_string.is_empty() {
        format!("{}<{}>", ast.ident, param_string)
    } else {
        ast.ident.to_string()
    }
}

/// This fetches the generics for impl blocks on the traits
/// and the where clause.
///
/// # Example:
/// ```
/// pub enum Enum<T: Debug, U>
///where
///     U: Into<T>
/// {
///     F1(T),
///     F2(U)
/// }
/// ```
/// returns `("<T: Debug, U>", "where U: Into<T>")`.
pub fn fetch_impl_generics(ast: &DeriveInput) -> (String, String) {
    let mut generics = ast.generics.clone();
    let where_clause = generics
        .where_clause
        .take()
        .map(|w| w.to_token_stream().to_string());
    (
        generics.to_token_stream().to_string(),
        where_clause.unwrap_or_default(),
    )
}

/// Fetches the name of each variant in the enum and
/// maps it to a string representation of its type.
///
/// Also performs validation for unsupported enum types.
/// These include:
///  * Enums with multiple variants of the same type.
///  * Enums with variants with multiple or named fields.
///  * Enums with unit variants.
///
/// Will panic if the input type is not an enum.
pub fn fetch_fields_from_enum(ast: &DeriveInput) -> HashMap<String, String> {
    if let Data::Enum(data) = &ast.data {
        data.variants
            .iter()
            .map(|var| match &var.fields {
                syn::Fields::Unnamed(field_) => {
                    if field_.unnamed.len() != 1 {
                        panic!(
                            "Can only derive for enums whose types do \
                             not contain multiple fields."
                        );
                    }
                    let field_ty = field_
                        .unnamed
                        .iter()
                        .next()
                        .unwrap()
                        .ty
                        .to_token_stream()
                        .to_string();
                    let field_name = var.ident.to_token_stream().to_string();
                    (field_name, field_ty)
                }
                syn::Fields::Named(_) => {
                    panic!("Can only derive for enums whose types do not have named fields.")
                }
                syn::Fields::Unit => {
                    panic!("Can only derive for enums who don't contain unit types as variants.")
                }
            })
            .collect()
    } else {
        panic!("Can only derive for enums.")
    }
}

/// Creates a marker enum for each field in the enum
/// under a new module.
///
/// Used to identify types in the enum and disambiguate
/// generic parameters.
pub fn create_marker_enums(name: &str, types: &HashMap<String, String>) -> TokenStream {
    let mut piece = format!("#[allow(non_snake_case]\n mod enum___conversion___{}", name);
    piece.push_str("{ ");
    for field in types.keys() {
        piece.push_str(&format!("pub(crate) enum {}{{}}", field));
    }
    piece.push('}');
    piece.parse().unwrap()
}

/// Get the fully qualified name of the marker struct
/// associated with an enum variant.
pub fn get_marker(name: &str, field: &str) -> String {
    format!("enum___conversion___{}::{}", name, field)
}


#[cfg(test)]
mod test_parsers {
    use super::*;

    #[test]
    fn test_parse_tuple_fields() {
        let ast: DeriveInput = syn::parse_str(
            r#"
            enum TupleTest {
                F1((i64, bool)),
            }
        "#,
        )
            .unwrap();
        let fields = fetch_fields_from_enum(&ast);
        let expected = HashMap::from([("F1".to_string(), "(i64 , bool)".to_string())]);
        assert_eq!(expected, fields);
    }

    #[test]
    fn test_parse_lifetime_fields() {
        let ast: DeriveInput = syn::parse_str(
            r#"
            enum TupleTest<'a> {
                F1((&'a i64, bool)),
            }
        "#,
        )
        .unwrap();
        let fields = fetch_fields_from_enum(&ast);
        let expected = HashMap::from([("F1".to_string(), "(& 'a i64 , bool)".to_string())]);
        assert_eq!(expected, fields);
    }

}