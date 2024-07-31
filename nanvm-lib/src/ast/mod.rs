use crate::js::any::Any;
use crate::mem::manager::Dealloc;
use std::string::String;

type Property<D> = (String, Expression<D>);

enum Expression<D: Dealloc> {
    LocalRef(u32),
    ArgRef(u32),
    Value(Any<D>),
    Object(Vec<Property<D>>),
    Array(Vec<Expression<D>>),
}

struct Body<D: Dealloc> {
    local: Vec<Expression<D>>,
    result: Expression<D>,
}

struct Module<D: Dealloc> {
    import: Vec<String>,
    body: Body<D>,
}
