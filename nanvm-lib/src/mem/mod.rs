mod arena;
pub mod block;
pub mod block_header;
mod constructor;
mod field_layout;
mod fixed;
pub mod flexible_array;
pub mod global;
pub mod local;
pub mod manager;
pub mod mut_ref;
pub mod object;
pub mod optional_block;
pub mod optional_ref;
pub mod ref_;
mod ref_counter_update;

#[cfg(test)]
mod test {
    use std::mem::{align_of, size_of};

    struct _MyStruct {
        a: u8,  // 1 byte
        b: u16, // 2 bytes
        c: u8,  // 1 byte
        d: u8,
    }

    const _: () = assert!(size_of::<_MyStruct>() == 6);
    const _: () = assert!(align_of::<_MyStruct>() == 2);
}
