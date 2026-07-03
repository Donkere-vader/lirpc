use crate::{
    translatable::{Translatable, Type},
    type_definition::{EnumDefinition, EnumVariant, EnumVariantFields, TypeDefinition},
};

pub trait LiRpcType
where
    Self: Translatable,
{
    fn translate() -> TypeDefinition;
}

impl<R, E> LiRpcType for Result<R, E>
where
    R: LiRpcType,
    E: LiRpcType,
{
    fn translate() -> TypeDefinition {
        TypeDefinition::Enum(Box::new(EnumDefinition {
            ident: "Result".to_string(),
            variants: vec![
                EnumVariant {
                    ident: "Ok".to_string(),
                    fields: EnumVariantFields::Unnamed(vec![Type::Generic("R".to_string())]),
                },
                EnumVariant {
                    ident: "Err".to_string(),
                    fields: EnumVariantFields::Unnamed(vec![Type::Generic("E".to_string())]),
                },
            ],
            generics: vec!["R".to_string(), "E".to_string()],
        }))
    }
}
