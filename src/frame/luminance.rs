#[inline]
pub fn rgb_luma_u8(r: u8, g: u8, b: u8) -> u8 {
    let r = r as u32;
    let g = g as u32;
    let b = b as u32;
    ((r * 77 + g * 150 + b * 29) >> 8) as u8
}

#[cfg(test)]
mod tests {
    use super::rgb_luma_u8;

    #[test]
    fn luma_for_black_and_white() {
        assert_eq!(rgb_luma_u8(0, 0, 0), 0);
        assert!(rgb_luma_u8(255, 255, 255) >= 254);
    }
}
