use ts_codegen::ast::{Generic, r#type::Type as TsType};

use crate::{
    translatable::Type,
    type_definition::{EnumDefinition, EnumVariant, EnumVariantFields},
};

pub fn free_generic(generics: &[Generic]) -> Generic {
    let mut n = 0;
    while generics.contains(&Generic(format!("T{n}"))) {
        n += 1;
    }

    Generic(format!("T{n}"))
}

pub fn rust_type_to_ts_type(rust_type: &Type) -> TsType {
    match rust_type {
        Type::I8
        | Type::I16
        | Type::I32
        | Type::I64
        | Type::I128
        | Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::U128 => TsType::Number,
        Type::String => TsType::String,
        Type::Bool => TsType::Boolean,
        Type::Generic(g) => TsType::Generic(Generic(g.to_string())),
        Type::TypeRef(ident) => TsType::TypeRef(ident.to_string()),
        Type::Vec(ty) => TsType::Array(Box::new(rust_type_to_ts_type(ty))),
        Type::Box(inner) => rust_type_to_ts_type(inner),
        Type::Option(_) => todo!(),
        Type::Result(_, _) => todo!(),
        Type::HashMap(t1, t2) => TsType::Class(
            "Map".to_string(),
            vec![rust_type_to_ts_type(t1), rust_type_to_ts_type(t2)],
        ),
        Type::Unit => TsType::Tuple(Vec::new()),
    }
}

pub fn gen_inner_type(enm: &EnumDefinition) -> TsType {
    match enm.variants.first() {
        Some(v) => TsType::Union(
            Box::new(variant_to_inner_type(v)),
            enm.variants
                .iter()
                .skip(1)
                .map(variant_to_inner_type)
                .collect(),
        ),
        None => TsType::Null,
    }
}

pub fn gen_variant_type(enm: &EnumDefinition) -> TsType {
    match enm.variants.first() {
        Some(v) => TsType::Union(
            Box::new(TsType::SpecificString(v.ident.to_string())),
            enm.variants
                .iter()
                .skip(1)
                .map(|v| TsType::SpecificString(v.ident.to_string()))
                .collect(),
        ),
        None => TsType::Null,
    }
}

pub fn variant_to_inner_type(EnumVariant { ident: _, fields }: &EnumVariant) -> TsType {
    match fields {
        EnumVariantFields::Unnamed(rust_types) => {
            TsType::Tuple(rust_types.iter().map(rust_type_to_ts_type).collect())
        }
        EnumVariantFields::Named(btree_map) => TsType::Object(
            btree_map
                .iter()
                .map(|(name, rust_type)| (name.to_string(), true, rust_type_to_ts_type(rust_type)))
                .collect(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use ts_codegen::to_ts::ToTS;

    use crate::{
        codegen::util::gen_inner_type,
        translatable::Type,
        type_definition::{EnumDefinition, EnumVariant},
    };

    #[test]
    fn test_gen_inner_type() {
        let enm = EnumDefinition {
            ident: "MyEnum".to_string(),
            generics: Vec::new(),
            variants: vec![
                EnumVariant::new_unit("A".to_string()),
                EnumVariant::new_tuple("B".to_string(), vec![Type::Bool, Type::I8]),
                EnumVariant::new_named("B".to_string(), vec![("a".to_string(), Type::Bool)]),
            ],
        };

        let actual = gen_inner_type(&enm).to_typescript();

        assert_eq!(actual, "[] | [boolean, number] | { a: boolean }")
    }
}
