pub trait StaticRefDefault: 'static {
    const STATIC_REF_DEFAULT: &'static Self;
}

impl<T: 'static> StaticRefDefault for Option<T> {
    const STATIC_REF_DEFAULT: &'static Self = &None;
}

impl StaticRefDefault for char {
    const STATIC_REF_DEFAULT: &'static Self = &(0 as char);
}
