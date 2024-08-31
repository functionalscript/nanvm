use std::rc::Rc;

use super::{Items, Primitive};

#[derive(Clone)]
pub enum Any {
    Primitive(Primitive),
    Number(f64),
    String(String),
    Array(Array),
    Object(Object),
    Bigint(Bigint),
}

type ArrayRc<T> = Rc<[T]>;

impl From<Primitive> for Any {
    fn from(value: Primitive) -> Self {
        Self::Primitive(value)
    }
}

impl From<f64> for Any {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<String> for Any {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<Object> for Any {
    fn from(value: Object) -> Self {
        Self::Object(value)
    }
}

impl From<Array> for Any {
    fn from(value: Array) -> Self {
        Self::Array(value)
    }
}

impl From<Bigint> for Any {
    fn from(value: Bigint) -> Self {
        Self::Bigint(value)
    }
}

impl<T> Items for ArrayRc<T> {
    type Item = T;
    fn items(&self) -> &[Self::Item] {
        self
    }
}

impl super::Any for Any {
    type Object = Object;
    type Array = Array;
    type String = String;
    type Bigint = Bigint;
    type Vm = Vm;
    fn switch<T: super::AnyMatch<Self>>(self, m: T) -> T::Result {
        match self {
            Any::Primitive(v) => m.primitive(v),
            Any::Number(v) => m.number(v),
            Any::String(v) => m.string(v),
            Any::Array(v) => m.array(v),
            Any::Object(v) => m.object(v),
            Any::Bigint(v) => m.bigint(v),
        }
    }
}

type String = ArrayRc<u16>;
type Array = ArrayRc<Any>;
type Object = ArrayRc<(String, Any)>;

#[derive(Clone)]
pub struct Bigint {
    negative: bool,
    rc: ArrayRc<u64>,
}

impl super::Items for Bigint {
    type Item = u64;
    fn items(&self) -> &[Self::Item] {
        &self.rc
    }
}

impl super::Bigint for Bigint {
    fn negative(&self) -> bool {
        self.negative
    }
}

type Vm = ();

impl super::Vm for Vm {
    type Any = Any;
    fn string(v: &[u16]) -> String {
        v.into()
    }
    fn array(v: &[Self::Any]) -> Array {
        v.into()
    }
    fn bigint(negative: bool, v: &[u64]) -> <Self::Any as super::Any>::Bigint {
        Bigint {
            negative,
            rc: v.into(),
        }
    }

    fn object(
        v: &[(<Self::Any as super::Any>::String, Self::Any)],
    ) -> <Self::Any as super::Any>::Object {
        v.into()
    }
}
