mod arena;
mod block;
mod field_layout;
mod fixed;
mod flexible_array;
mod global;
mod local;
pub mod manager;
pub mod mut_ref;
mod new_in_place;
mod object;
mod optional_block;
mod optional_ref;
mod ref_;
mod ref_counter_update;

#[cfg(test)]
mod test {
    use std::mem::{align_of, size_of};

    struct MyStruct {
        a: u8,  // 1 byte
        b: u16, // 2 bytes
        c: u8,  // 1 byte
        d: u8,
    }

    const _: () = assert!(size_of::<MyStruct>() == 6);
    const _: () = assert!(align_of::<MyStruct>() == 2);
}
