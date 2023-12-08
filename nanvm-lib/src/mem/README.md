# Memory Management

## Glossary

- `manager` a structure which manages a block of memory.
- `object` an object which can be stored in dynamic memory. Each object should implement the [Object](./object.rs#L10) trait.
- `block` an area of memory which can store an object. Each block contains a `header` and an `object`.

## Reference Types

- `MutRef<T>` can't be cloned. It's not a reference counter.
  ```rust
  impl<T> Deref for MutRef<T>;
  impl<T> DerefMut for MutRef<T>;
  impl<T> MutRef {
    // we need to pass ownership of `MutRef<T>` to the caller.
    pub fn to_ref(self) -> Ref<T>;
  }
  ```
- `Ref<T>` can be cloned. It's a reference counter. If we own `Ref<T>` and the number of references is `0`, we can get a `MutRef<T>` from it, otherwise we will have to create a new object `T`.
  ```rust
  impl<T> Clone for Ref<T>;
  impl<T> Deref for Ref<T>;
  impl<T> Ref {
    // we need to pass ownership of `Ref<T>` to the caller.
    pub fn try_to_mut_ref(self) -> Result<MutRef<T>, Self> {
        if self.ref_count() == 0 {
            let ptr = self.ptr;
            forget(self);
            Ok(MutRef { ptr })
        } else {
            Err(self)
        }
    }
    pub fn clone_object_from_ref(&self) -> MutRef<T>;
    // we need to pass ownership of `Ref<T>` to the caller.
    pub fn to_mut_ref(self) -> MutRef<T> {
        match self.get_mut_ref() {
            Ok(x) => x,
            Err(x) => x.clone_object_from_ref(),
        }
    }
  }
  ```

## Notes

If you implement a mock version of a reference counter, it should return a number which is greater than `0` when calling `ref_count()`. Otherwise, `try_get_mut_ref()` will assume that it has an exclusive ownership of the object, which is not true in general.

## Future Changes

Each manager should have a type `Dealloc`. In this case, `Ref` and `MutRef` will depend on the type instead of a complete manager. We need to be able to pass a manager as `&mut` to `new` without taking a complete ownership of the manager. This feature will be useful for single-threaded allocators because we don't need to use `Cell` for arenas.
