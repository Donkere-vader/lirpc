use crate::{
    ast::{Visibility, r#type::Type},
    to_ts::ToTS,
};

#[derive(Debug)]
pub struct ClassMember {
    pub visibility: Visibility,
    pub ident: String,
    pub r#type: Type,
    pub r#static: bool,
}

impl ToTS for ClassMember {
    fn to_typescript(&self) -> String {
        format!(
            "{}{}{}: {}",
            self.visibility.to_typescript(),
            if self.r#static { " static " } else { " " },
            self.ident,
            self.r#type.to_typescript()
        )
        .trim()
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{Visibility, class::class_member::ClassMember, r#type::Type},
        to_ts::ToTS,
    };

    #[test]
    fn test_private_class_member_to_typescript() {
        let member = ClassMember {
            visibility: Visibility::Private,
            ident: "var0".to_string(),
            r#type: Type::String,
            r#static: false,
        };

        let actual = member.to_typescript();

        assert_eq!(actual, "private var0: string")
    }

    #[test]
    fn test_public_static_class_member_to_typescript() {
        let member = ClassMember {
            visibility: Visibility::Public,
            ident: "var0".to_string(),
            r#type: Type::String,
            r#static: true,
        };

        let actual = member.to_typescript();

        assert_eq!(actual, "public static var0: string")
    }
}
