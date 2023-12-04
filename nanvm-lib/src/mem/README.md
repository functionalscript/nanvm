# Memory Management

## Glossary

- `manager` a structure which manages a block of memory.
- `object` an object which can be stored in dynamic memory. Each object should implement the [Object](./object.rs#L10) trait.
- `block` an area of memory which can store an object. Each block contains a `header` and an `object`.

## Reference Types

- `MutRef<T>` can't be cloned. It's not a reference counter.
  ```rust
  impl<T> MutRef {
    pub fn get_object(&self) -> &T;
    pub fn get_mut_object(&mut self) -> &mut T;
    // we need to pass ownership of `MutRef<T>` to the caller.
    pub fn get_ref(self) -> Ref<T>;
  }
  ```
- `Ref<T>` can be cloned. It's a reference counter. If we own `Ref<T>` and the number of references is 1, we can get a `MutRef<T>` from it, otherwise we will have to create a new object `T`.
  ```rust
  impl<T> Clone for Ref<T>;
  impl<T> Ref {
    pub fn get_object(&self) -> &T;
    // we need to pass ownership of `Ref<T>` to the caller.
    pub fn try_get_mut_ref(self) -> Result<MutRef<T>, Self> {
        if self.ref_count() == 1 {
            Ok(...)
        } else {
            Err(self)
        }
    }
    pub fn clone_object_from_ref(&self) -> MutRef<T>;
    // we need to pass ownership of `Ref<T>` to the caller.
    pub fn clone_object(self) -> MutRef<T> {
        match self.try_get_mut_ref() {
            Ok(x) => x,
            Err(x) => x.clone_object_from_ref(),
        }
    }
  }
  ```
