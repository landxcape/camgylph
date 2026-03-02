#[inline]
pub fn sample_source_coords(
    x: u16,
    y: u16,
    out_width: u16,
    out_height: u16,
    src_width: u32,
    src_height: u32,
) -> (u32, u32) {
    let sx = ((x as u32 * src_width) / out_width as u32).min(src_width.saturating_sub(1));
    let sy = ((y as u32 * src_height) / out_height as u32).min(src_height.saturating_sub(1));
    (sx, sy)
}

#[cfg(test)]
mod tests {
    use super::sample_source_coords;

    #[test]
    fn sample_coords_stay_within_bounds() {
        let (sx, sy) = sample_source_coords(79, 23, 80, 24, 640, 480);
        assert!(sx < 640);
        assert!(sy < 480);
    }
}
