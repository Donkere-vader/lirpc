use ts_codegen::ast::{Visibility, class::class_member::ClassMember};

use crate::{
    codegen::util::{gen_inner_type, gen_variant_type},
    type_definition::EnumDefinition,
};

pub fn gen_class_members_for_enum(enm: &EnumDefinition) -> Vec<ClassMember> {
    let variant_member = ClassMember {
        visibility: Visibility::Private,
        ident: "variant".to_string(),
        r#type: gen_variant_type(enm),
        r#static: false,
    };

    let inner_member = ClassMember {
        visibility: Visibility::Private,
        ident: "inner".to_string(),
        r#type: gen_inner_type(enm),
        r#static: false,
    };

    vec![variant_member, inner_member]
}
