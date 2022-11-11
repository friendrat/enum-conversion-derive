pub(crate) const GET_VARIANT_TEMPLATE: &str = r#"
impl{{ generics }} variant_access_traits::GetVariant<{{ Type }}, {{ Marker }} > for {{ fullname }}
{{ Where }} {
    fn get_variant(self) -> std::result::Result<{{ Type }}, variant_access_traits::VariantAccessError> {
        match self {
            {{ name }}::{{ field }}(inner) => Ok(inner),
            _ => Err(variant_access_traits::VariantAccessError::wrong_active_field("{{ fullname }}", "{{ Type }}"))
        }
    }

    fn get_variant_ref(&self) -> std::result::Result<&{{ Type }}, variant_access_traits::VariantAccessError> {
        match &self {
            {{ name }}::{{ field }}(inner) => Ok(inner),
            _ => Err(variant_access_traits::VariantAccessError::wrong_active_field("{{ fullname }}", "{{ Type }}"))
        }
    }

    fn get_variant_mut(&mut self) -> std::result::Result<&mut {{ Type }}, variant_access_traits::VariantAccessError> {
        match self {
            {{ name }}::{{ field }}(inner) => Ok(inner),
            _  => Err(variant_access_traits::VariantAccessError::wrong_active_field("{{ fullname }}", "{{ Type }}"))
        }
    }
"#;

pub(crate) const TRY_FROM_TEMPLATE: &str = r#"
impl{{ generics }} TryFrom<{{ fullname }}> for {{ Type }}
{{ Where }}
{
    type Error = Box<dyn Error + 'static>

    fn try_from(value: {{ name }}) -> std::result::Result<Self, Self::Error> {
        value.get_variant().map_err(|e| e.to_string().into())
    }
}"#;

pub(crate) const FROM_TEMPLATE: &str = r#"
impl{{ generics }} From<{{ Type }}> for {{ fullname }}
{{ Where }}
{

    fn from(value: {{ Type }}) -> Self {
        Self::{{ field }}(value)
    }
}
"#;
