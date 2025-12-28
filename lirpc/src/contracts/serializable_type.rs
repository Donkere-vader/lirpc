use serde::{Deserialize, Serialize};
use syn::{
    Expr, ExprLit, GenericArgument, Lit, PathArguments, Type, TypeArray, TypePath, TypeTuple,
};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SerializableType {
    Array {
        children: Box<SerializableType>,
        length: usize,
    },
    Vec {
        children: Box<SerializableType>,
    },
    Custom {
        name: String,
    },
    Result {
        ok: Box<SerializableType>,
        err: Box<SerializableType>,
    },
    Option {
        some: Box<SerializableType>,
    },
    Tuple {
        children: Vec<SerializableType>,
    },
    Bool,
    String,
    U128,
    U64,
    U32,
    U16,
    U8,
    I128,
    I64,
    I32,
    I16,
    I8,
    F64,
    F32,
}

impl TryFrom<Type> for SerializableType {
    type Error = String;

    fn try_from(value: Type) -> Result<Self, Self::Error> {
        match value {
            Type::Array(TypeArray { elem, len, .. }) => {
                // Only support literal array lengths
                let length = match len {
                    Expr::Lit(ExprLit {
                        lit: Lit::Int(ref int_lit),
                        ..
                    }) => int_lit
                        .base10_parse::<usize>()
                        .map_err(|e| format!("Invalid array length: {}", e))?,
                    _ => {
                        return Err(
                            "Unsupported array length expression, expected integer literal".into(),
                        );
                    }
                };
                let child = SerializableType::try_from(*elem)?;
                Ok(SerializableType::Array {
                    children: Box::new(child),
                    length,
                })
            }
            Type::Tuple(TypeTuple { elems, .. }) => {
                let mut children = Vec::with_capacity(elems.len());
                for elem in elems {
                    let child = SerializableType::try_from(elem)?;
                    children.push(child);
                }
                Ok(SerializableType::Tuple { children })
            }
            Type::Path(TypePath { path, .. }) => {
                // Extract last segment for type name and look at generic args
                let last = path
                    .segments
                    .last()
                    .ok_or_else(|| "Empty type path".to_string())?;
                let ident = last.ident.to_string();

                // Helper to get generic type arguments from angle brackets, if any
                let angle_args = match &last.arguments {
                    PathArguments::AngleBracketed(abga) => Some(&abga.args),
                    _ => None,
                };

                match ident.as_str() {
                    // Primitives
                    "bool" => Ok(SerializableType::Bool),
                    "String" => Ok(SerializableType::String),
                    "u128" => Ok(SerializableType::U128),
                    "u64" => Ok(SerializableType::U64),
                    "u32" => Ok(SerializableType::U32),
                    "u16" => Ok(SerializableType::U16),
                    "u8" => Ok(SerializableType::U8),
                    "i128" => Ok(SerializableType::I128),
                    "i64" => Ok(SerializableType::I64),
                    "i32" => Ok(SerializableType::I32),
                    "i16" => Ok(SerializableType::I16),
                    "i8" => Ok(SerializableType::I8),
                    "f64" => Ok(SerializableType::F64),
                    "f32" => Ok(SerializableType::F32),
                    // Containers
                    "Vec" => {
                        let args = angle_args
                            .ok_or_else(|| "Vec must have one generic argument".to_string())?;
                        // Expect single type arg
                        let mut iter = args.iter().filter_map(|ga| {
                            if let GenericArgument::Type(t) = ga {
                                Some(t.clone())
                            } else {
                                None
                            }
                        });
                        let child_ty = iter
                            .next()
                            .ok_or_else(|| "Vec missing type argument".to_string())?;
                        if iter.next().is_some() {
                            return Err("Vec must have exactly one type argument".into());
                        }
                        let child = SerializableType::try_from(child_ty)?;
                        Ok(SerializableType::Vec {
                            children: Box::new(child),
                        })
                    }
                    "Option" => {
                        let args = angle_args
                            .ok_or_else(|| "Option must have one generic argument".to_string())?;
                        let mut iter = args.iter().filter_map(|ga| {
                            if let GenericArgument::Type(t) = ga {
                                Some(t.clone())
                            } else {
                                None
                            }
                        });
                        let child_ty = iter
                            .next()
                            .ok_or_else(|| "Option missing type argument".to_string())?;
                        if iter.next().is_some() {
                            return Err("Option must have exactly one type argument".into());
                        }
                        let child = SerializableType::try_from(child_ty)?;
                        Ok(SerializableType::Option {
                            some: Box::new(child),
                        })
                    }
                    "Result" => {
                        let args = angle_args
                            .ok_or_else(|| "Result must have two generic arguments".to_string())?;
                        let mut iter = args.iter().filter_map(|ga| {
                            if let GenericArgument::Type(t) = ga {
                                Some(t.clone())
                            } else {
                                None
                            }
                        });
                        let ok_ty = iter
                            .next()
                            .ok_or_else(|| "Result missing Ok type argument".to_string())?;
                        let err_ty = iter
                            .next()
                            .ok_or_else(|| "Result missing Err type argument".to_string())?;
                        if iter.next().is_some() {
                            return Err("Result must have exactly two type arguments".into());
                        }
                        let ok = SerializableType::try_from(ok_ty)?;
                        let err = SerializableType::try_from(err_ty)?;
                        Ok(SerializableType::Result {
                            ok: Box::new(ok),
                            err: Box::new(err),
                        })
                    }
                    // Anything else is treated as a custom type
                    _ => {
                        let name = last.ident.to_string();
                        Ok(SerializableType::Custom { name })
                    }
                }
            }
            // Unsupported syn::Type variants
            other => Err(format!("Unsupported type: {:?}", other)),
        }
    }
}
