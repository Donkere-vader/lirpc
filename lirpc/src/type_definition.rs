use serde::{Deserialize, Serialize};

use crate::translatable::Type;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnumDefinition {
    pub ident: String,
    pub variants: Vec<EnumVariant>,
    pub generics: Vec<String>,
}

impl EnumDefinition {
    pub fn new(ident: String, variants: Vec<EnumVariant>, generics: Vec<String>) -> Self {
        Self {
            ident,
            variants,
            generics,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnumVariant {
    pub ident: String,
    pub fields: EnumVariantFields,
}

impl EnumVariant {
    pub fn new(ident: String, fields: EnumVariantFields) -> Self {
        Self { ident, fields }
    }

    pub fn new_unit(ident: String) -> Self {
        Self {
            ident,
            fields: EnumVariantFields::Unnamed(Vec::new()),
        }
    }

    pub fn new_tuple(ident: String, types: Vec<Type>) -> Self {
        Self {
            ident,
            fields: EnumVariantFields::Unnamed(types),
        }
    }

    pub fn new_named(ident: String, fields: Vec<(String, Type)>) -> Self {
        Self {
            ident,
            fields: EnumVariantFields::Named(fields),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum EnumVariantFields {
    Named(Vec<(String, Type)>),
    Unnamed(Vec<Type>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StructDefinition {
    pub ident: String,
    pub fields: StructFields,
    pub generics: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum StructFields {
    Named(Vec<(String, Type)>),
    Unnamed(Vec<Type>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TypeDefinition {
    Enum(Box<EnumDefinition>),
    Struct(Box<StructDefinition>),
}

impl TypeDefinition {
    pub fn get_type_ident(&self) -> &str {
        match self {
            Self::Enum(enm) => &enm.ident,
            Self::Struct(strct) => &strct.ident,
        }
    }
}
