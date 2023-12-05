/// Update a reference count.
pub enum RefUpdate {
    AddRef = 1,
    Read = 0,
    Release = -1,
}
