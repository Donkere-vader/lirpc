use crate::{
    ast::{assign_to::AssignTo, statement::expression_statement::ExpressionStatement},
    to_ts::ToTS,
};

#[derive(Debug)]
pub enum AssignmentType {
    Const,
    Let,
    Var,
    Existing,
}

#[derive(Debug)]
pub struct AssignmentStatement {
    pub kind: AssignmentType,
    pub left_hand_side: AssignTo,
    pub right_hand_side: ExpressionStatement,
}

impl ToTS for AssignmentStatement {
    fn to_typescript(&self) -> String {
        let kind_str = match self.kind {
            AssignmentType::Const => "const ",
            AssignmentType::Let => "let ",
            AssignmentType::Var => "var ",
            AssignmentType::Existing => "",
        };

        format!(
            "{kind_str}{} = {}",
            self.left_hand_side.to_typescript(),
            self.right_hand_side.to_typescript()
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{assign_to::AssignTo, statement::expression_statement::ExpressionStatement},
        to_ts::ToTS,
    };

    use super::{AssignmentStatement, AssignmentType};

    #[test]
    fn test_const_assignment_to_typescript() {
        let ass = AssignmentStatement {
            kind: AssignmentType::Const,
            left_hand_side: AssignTo::Ident("my_const".to_string()),
            right_hand_side: ExpressionStatement::RawStatement("\"Hello, world!\"".to_string()),
        };

        let actual = ass.to_typescript();

        assert_eq!(actual, "const my_const = \"Hello, world!\"");
    }

    #[test]
    fn test_let_assignment_to_typescript() {
        let ass = AssignmentStatement {
            kind: AssignmentType::Let,
            left_hand_side: AssignTo::UnpackArray(vec![
                AssignTo::Ident("a".to_string()),
                AssignTo::Ident("b".to_string()),
            ]),
            right_hand_side: ExpressionStatement::RawStatement(
                "[12, \"Hello, world!\"]".to_string(),
            ),
        };

        let actual = ass.to_typescript();

        assert_eq!(actual, "let [a, b] = [12, \"Hello, world!\"]");
    }

    #[test]
    fn test_var_assignment_to_typescript() {
        let ass = AssignmentStatement {
            kind: AssignmentType::Var,
            left_hand_side: AssignTo::UnpackObject(vec![
                ("a".to_string(), AssignTo::Ident("a".to_string())),
                ("b".to_string(), AssignTo::Ident("b".to_string())),
            ]),
            right_hand_side: ExpressionStatement::RawStatement(
                "{ a: 12, b: \"Hello, world!\" }".to_string(),
            ),
        };

        let actual = ass.to_typescript();

        assert_eq!(actual, "var { a, b } = { a: 12, b: \"Hello, world!\" }");
    }

    #[test]
    fn test_existing_assignment_to_typescript() {
        let ass = AssignmentStatement {
            kind: AssignmentType::Existing,
            left_hand_side: AssignTo::Ident("something".to_string()),
            right_hand_side: ExpressionStatement::RawStatement("\"Hello, world!\"".to_string()),
        };

        let actual = ass.to_typescript();

        assert_eq!(actual, "something = \"Hello, world!\"");
    }
}
