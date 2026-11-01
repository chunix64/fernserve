use gcd::binary_usize;

pub const fn lcm(a: usize, b: usize) -> usize {
    if a == 0 || b == 0 {
        return 0;
    }
    a * (b / binary_usize(a, b))
}
