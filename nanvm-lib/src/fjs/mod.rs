use core::result;

mod simple;

#[derive(Clone)]
enum Primitive {
    Null = 0,
    Undefined = 1,
    True = 2,
    False = 3,
}

trait AnyMatch<T: Any> {
    type Result;
    //
    fn primitive(self, v: Primitive) -> Self::Result;
    fn number(self, v: f64) -> Self::Result;
    //
    fn object(self, v: T::Object) -> Self::Result;
    fn array(self, v: T::Array) -> Self::Result;
    fn string(self, v: T::String) -> Self::Result;
    fn bigint(self, v: T::Bigint) -> Self::Result;
}

trait Items {
    type Item;
    fn items(&self) -> &[Self::Item];
}

trait Bigint: Items<Item = u64> {
    fn negative(&self) -> bool;
}


enum Error {
    OutOfMemory = 1
}

type Result<T> = result::Result<T, Error>;

trait Any:
    Sized
    + From<Primitive>
    + From<f64>
    + From<Self::String>
    + From<Self::Object>
    + From<Self::Array>
    + From<Self::Bigint>
{
    type Vm: Vm<Any = Self>;
    type Object: Items<Item = (Self::String, Self)>;
    type Array: Items<Item = Self>;
    type String: Items<Item = u16>;
    type Bigint: Bigint;
    fn switch<T: AnyMatch<Self>>(self, m: T) -> T::Result;
}

trait Vm {
    type Any: Any<Vm = Self>;
    fn string(v: &[u16]) -> Result<<Self::Any as Any>::String>;
    fn array(v: &[Self::Any]) -> Result<<Self::Any as Any>::Array>;
    fn object(
        v: &[(<Self::Any as Any>::String, Self::Any)],
    ) -> Result<<Self::Any as Any>::Object>;
    fn bigint(negative: bool, v: &[u64]) -> Result<<Self::Any as Any>::Bigint>;
}
