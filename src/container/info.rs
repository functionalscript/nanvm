use crate::allocator::Allocator;

pub trait Info: Sized {
    type Item;
    type Allocator: Allocator;
}
