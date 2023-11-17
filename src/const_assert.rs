#[inline(always)]
pub const fn const_assert(v: bool) -> () {
    #[allow(clippy::erasing_op)]
    let _ = 0 / v as usize;
}
