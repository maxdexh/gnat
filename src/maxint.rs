//! Implementation detail

pub(crate) type Umax = crate::condty::CondTy<
    crate::consts::Bool<{ size_of::<usize>() > size_of::<u128>() }>,
    usize,
    u128,
>;

pub(crate) const fn umax_strlen(n: Umax) -> usize {
    if n == 0 { 1 } else { n.ilog10() as usize + 1 }
}
pub(crate) const fn umax_write(n: Umax, out: &mut [u8]) -> &mut [u8] {
    let (mut n_out, out) = out.split_at_mut(umax_strlen(n));
    let mut r = n;
    while let [rem @ .., last] = n_out {
        n_out = rem;
        *last = b'0' + (r % 10) as u8;
        r /= 10;
        if r == 0 {
            break;
        }
    }
    debug_assert!(r == 0);
    out
}
