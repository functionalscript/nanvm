#[inline(always)]
pub const fn const_assert(v: bool) -> () {
    let _ = 0 / v as usize;
}
