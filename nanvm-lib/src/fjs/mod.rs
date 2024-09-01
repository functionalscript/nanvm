use std::rc::Rc;

type ArrayRc<T> = Rc<[T]>;

// "hello"
type String = ArrayRc<u16>;

// [6, false]
type Array = ArrayRc<Any>;

// { x: "d" }
type Object = ArrayRc<(String, Any)>;

// 34n
type Bigint = ArrayRc<u64>;

// () => 5
type Function = Rc<Any>;

enum Any {
    // void 0
    Undefined,
    // null
    Null,
    // false
    Bool(bool),
    // 3.14
    Number(f64),
    // "Hello world!"
    String(String),
    // [43, true]
    Array(Array),
    // { a: "H", b: 5 }
    Object(Object),
    // 42n
    Bigint(Bigint),
    // () => () => 5
    Function(Function),
}
