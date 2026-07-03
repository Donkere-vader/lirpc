use crate::{
    ast::{
        class::ClassDefinition,
        statement::{
            assignment_statement::AssignmentStatement, export_statement::ExportStatement,
            expression_statement::ExpressionStatement, for_statement::ForStatement,
            if_statement::IfStatement, return_statement::ReturnStatement,
            switch_case_statement::SwitchCaseStatement, type_definition::TypeDefinitionStatement,
            while_statement::WhileStatement,
        },
    },
    to_ts::ToTS,
};

pub mod assignment_statement;
pub mod export_statement;
pub mod expression_statement;
pub mod for_statement;
pub mod if_statement;
pub mod return_statement;
pub mod switch_case_statement;
pub mod type_definition;
pub mod while_statement;

#[derive(Debug)]
pub enum Statement {
    If(IfStatement),
    While(WhileStatement),
    For(ForStatement),
    Expression(ExpressionStatement),
    Assignment(AssignmentStatement),
    ReturnStatement(ReturnStatement),
    BreakStatement,
    SwitchCase(SwitchCaseStatement),
    ClassDefinition(ClassDefinition),
    TypeDefinition(TypeDefinitionStatement),
    ExportStatement(ExportStatement),
}

impl ToTS for Statement {
    fn to_typescript(&self) -> String {
        match self {
            Self::If(s) => s.to_typescript(),
            Self::While(s) => s.to_typescript(),
            Self::For(s) => s.to_typescript(),
            Self::Expression(s) => s.to_typescript(),
            Self::Assignment(s) => s.to_typescript(),
            Self::ReturnStatement(s) => s.to_typescript(),
            Self::BreakStatement => "break".to_string(),
            Self::SwitchCase(s) => s.to_typescript(),
            Self::ClassDefinition(c) => c.to_typescript(),
            Self::TypeDefinition(t) => t.to_typescript(),
            Self::ExportStatement(s) => s.to_typescript(),
        }
    }
}
