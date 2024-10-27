trait FromExactSizeIterator<T> {
    fn from(v: impl ExactSizeIterator<Item = T>) -> Self;
}

trait String: FromExactSizeIterator<u16> {}

trait Object: FromExactSizeIterator<Self::Any> {
    type Any: Any;
}

type Property<T> = (<T as Any>::String, T);

trait Array: FromExactSizeIterator<Property<Self::Any>> {
    type Any: Any;
}

trait Bigint: From<i128> {}

trait Function {}

enum None {
    Undefined,
    Null,
}

trait Any: From<None> + From<bool> + From<f64> {
    type String: String;
    type Object: Object<Any = Self>;
    type Array: Array<Any = Self>;
    type Bigint: Bigint;
    type Function: Function;
}
