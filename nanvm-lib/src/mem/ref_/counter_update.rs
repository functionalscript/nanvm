/// Update a reference count.
pub enum RefCounterUpdate {
    AddRef = 1,
    Read = 0,
    Release = -1,
}
