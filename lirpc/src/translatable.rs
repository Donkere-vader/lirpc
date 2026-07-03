use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum Type {
    TypeRef(String),
    Generic(String),
    Box(Box<Type>),
    Vec(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Option(Box<Type>),
    HashMap(Box<Type>, Box<Type>),
    Unit,
    String,
    Bool,
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
}

pub trait Translatable
where
    Self: Serialize + for<'a> Deserialize<'a>,
{
    fn get_type() -> Type;
}

macro_rules! impl_basic_translatable {
    ($Ti:ty, $to:expr) => {
        impl Translatable for $Ti {
            fn get_type() -> Type {
                $to
            }
        }
    };
}

impl_basic_translatable!(i8, Type::I8);
impl_basic_translatable!(i16, Type::I16);
impl_basic_translatable!(i32, Type::I32);
impl_basic_translatable!(i64, Type::I64);
impl_basic_translatable!(i128, Type::I128);
impl_basic_translatable!(u8, Type::U8);
impl_basic_translatable!(u16, Type::U16);
impl_basic_translatable!(u32, Type::U32);
impl_basic_translatable!(u64, Type::U64);
impl_basic_translatable!(u128, Type::U128);
impl_basic_translatable!(bool, Type::Bool);
impl_basic_translatable!(String, Type::String);
impl_basic_translatable!((), Type::Unit);

impl<T: Translatable> Translatable for Box<T> {
    fn get_type() -> Type {
        Type::Box(Box::new(T::get_type()))
    }
}

impl<R: Translatable, E: Translatable> Translatable for Result<R, E> {
    fn get_type() -> Type {
        Type::Result(Box::new(R::get_type()), Box::new(E::get_type()))
    }
}

impl<T: Translatable> Translatable for Option<T> {
    fn get_type() -> Type {
        Type::Option(Box::new(T::get_type()))
    }
}

impl<T: Translatable> Translatable for Vec<T> {
    fn get_type() -> Type {
        Type::Vec(Box::new(T::get_type()))
    }
}
