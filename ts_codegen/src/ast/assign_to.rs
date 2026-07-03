use crate::to_ts::ToTS;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AssignTo {
    Ident(String),
    UnpackObject(Vec<(String, AssignTo)>),
    UnpackArray(Vec<AssignTo>),
}

impl AssignTo {
    fn is_ident(&self, ident: &str) -> bool {
        match self {
            Self::Ident(i) => ident == i,
            _ => false,
        }
    }
}

impl ToTS for AssignTo {
    fn to_typescript(&self) -> String {
        match self {
            Self::Ident(ident) => ident.to_string(),
            Self::UnpackObject(fields) => format!(
                "{{ {} }}",
                fields
                    .iter()
                    .map(|(name, assignment)| if assignment.is_ident(name) {
                        name.to_string()
                    } else {
                        format!("{name}: {}", assignment.to_typescript())
                    })
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::UnpackArray(items) => format!(
                "[{}]",
                items
                    .iter()
                    .map(ToTS::to_typescript)
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{ast::assign_to::AssignTo, to_ts::ToTS};

    #[test]
    fn test_assign_to_ident_to_typescript() {
        let assign_to = AssignTo::Ident("var0".to_string());

        let actual = assign_to.to_typescript();

        assert_eq!(actual, "var0");
    }

    #[test]
    fn test_assign_to_unpack_object_to_typescript() {
        let assign_to = AssignTo::UnpackObject(vec![
            ("a".to_string(), AssignTo::Ident("a".to_string())),
            ("b".to_string(), AssignTo::Ident("renamed".to_string())),
        ]);

        let actual = assign_to.to_typescript();

        assert_eq!(actual, "{ a, b: renamed }");
    }

    #[test]
    fn test_assign_to_unpack_array_to_typescript() {
        let assign_to = AssignTo::UnpackArray(vec![
            AssignTo::Ident("a".to_string()),
            AssignTo::Ident("b".to_string()),
            AssignTo::UnpackArray(vec![AssignTo::Ident("c".to_string())]),
        ]);

        let actual = assign_to.to_typescript();

        assert_eq!(actual, "[a, b, [c]]");
    }
}
