pub fn fit_dimensions(
    src_width: u32,
    src_height: u32,
    term_width: u16,
    term_height: u16,
    cell_aspect_ratio: f32,
) -> (u16, u16) {
    let max_w = term_width.max(1);
    let max_h = term_height.saturating_sub(1).max(1);

    let src_aspect = src_width as f32 / src_height as f32;

    let mut out_w = max_w;
    let mut out_h = ((out_w as f32 / src_aspect) * cell_aspect_ratio).round() as u16;
    out_h = out_h.max(1);

    if out_h > max_h {
        out_h = max_h;
        out_w = ((out_h as f32 * src_aspect) / cell_aspect_ratio).round() as u16;
        out_w = out_w.clamp(1, max_w);
    }

    (out_w, out_h)
}

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
    use super::{fit_dimensions, sample_source_coords};

    #[test]
    fn fit_dimensions_respects_terminal_bounds() {
        let (w, h) = fit_dimensions(640, 480, 120, 40, 0.5);
        assert!(w <= 120);
        assert!(h <= 39);
        assert!(w >= 1);
        assert!(h >= 1);
    }

    #[test]
    fn sample_coords_stay_within_bounds() {
        let (sx, sy) = sample_source_coords(79, 23, 80, 24, 640, 480);
        assert!(sx < 640);
        assert!(sy < 480);
    }
}
