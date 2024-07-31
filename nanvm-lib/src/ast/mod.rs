use crate::js::any::Any;
use crate::mem::{global::Global, manager::Dealloc};
use std::string::String;

type Property<D = Global> = (String, Expression<D>);

enum Expression<D: Dealloc = Global> {
    LocalRef(u32),
    ArgRef(u32),
    Value(Any<D>),
    Object(Vec<Property<D>>),
    Array(Vec<Expression<D>>),
}

struct Body<D: Dealloc = Global> {
    local: Vec<Expression<D>>,
    result: Expression<D>,
}

struct Module<D: Dealloc = Global> {
    import: Vec<String>,
    body: Body<D>,
}
