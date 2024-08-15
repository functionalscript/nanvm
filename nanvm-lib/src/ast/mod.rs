use crate::common::default::default;
use crate::js::{any::Any, js_string::JsStringRef};
use crate::mem::manager::Dealloc;

pub type Property<D> = (JsStringRef<D>, Expression<D>);

#[derive(Default)]
pub enum Expression<D: Dealloc> {
    #[default]
    Void,
    LocalRef(u32),
    ArgRef(u32),
    Value(Any<D>),
    Object(Vec<Property<D>>),
    Array(Vec<Expression<D>>),
}

pub struct Body<D: Dealloc> {
    local: Vec<Expression<D>>,
    result: Expression<D>,
}

impl<D: Dealloc> Default for Body<D> {
    fn default() -> Self {
        Self {
            local: default(),
            result: default(),
        }
    }
}

pub struct Module<D: Dealloc> {
    import: Vec<JsStringRef<D>>,
    body: Body<D>,
}

impl<D: Dealloc> Default for Module<D> {
    fn default() -> Self {
        Self {
            import: Vec::new(),
            body: default(),
        }
    }
}
