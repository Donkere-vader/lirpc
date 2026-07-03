use crate::to_ts::ToTS;

pub mod assign_to;
pub mod class;
pub mod code_block;
pub mod statement;
pub mod r#type;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Generic(pub String);

impl ToTS for Generic {
    fn to_typescript(&self) -> String {
        self.0.to_string()
    }
}

impl ToTS for Vec<Generic> {
    fn to_typescript(&self) -> String {
        if !self.is_empty() {
            format!(
                "<{}>",
                self.iter()
                    .map(ToTS::to_typescript)
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        } else {
            String::new()
        }
    }
}

#[derive(Debug)]
pub enum Visibility {
    Public,
    Private,
    Protected,
}

impl ToTS for Visibility {
    fn to_typescript(&self) -> String {
        match self {
            Self::Public => "public",
            Self::Private => "private",
            Self::Protected => "protected",
        }
        .to_string()
    }
}
