#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoverCrop {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub fn compute_cover_crop(
    src_width: u32,
    src_height: u32,
    out_width: u16,
    out_height: u16,
    cell_aspect_ratio: f32,
) -> CoverCrop {
    let out_w = out_width.max(1) as f32;
    let out_h = out_height.max(1) as f32;
    let cell = cell_aspect_ratio.max(0.01);

    let source_aspect = src_width as f32 / src_height as f32;
    let target_aspect = (out_w / out_h) * cell;

    if source_aspect > target_aspect {
        let crop_h = src_height;
        let crop_w = ((src_height as f32 * target_aspect).round() as u32).clamp(1, src_width);
        let crop_x = (src_width.saturating_sub(crop_w)) / 2;
        return CoverCrop {
            x: crop_x,
            y: 0,
            width: crop_w,
            height: crop_h,
        };
    }

    let crop_w = src_width;
    let crop_h = ((src_width as f32 / target_aspect).round() as u32).clamp(1, src_height);
    let crop_y = (src_height.saturating_sub(crop_h)) / 2;
    CoverCrop {
        x: 0,
        y: crop_y,
        width: crop_w,
        height: crop_h,
    }
}

#[inline]
pub fn sample_source_coords_cover(
    x: u16,
    y: u16,
    out_width: u16,
    out_height: u16,
    src_width: u32,
    src_height: u32,
    cell_aspect_ratio: f32,
) -> (u32, u32) {
    let crop = compute_cover_crop(
        src_width,
        src_height,
        out_width,
        out_height,
        cell_aspect_ratio,
    );

    let sx = crop.x
        + ((x as u32 * crop.width) / out_width.max(1) as u32).min(crop.width.saturating_sub(1));
    let sy = crop.y
        + ((y as u32 * crop.height) / out_height.max(1) as u32).min(crop.height.saturating_sub(1));
    (sx, sy)
}

#[cfg(test)]
mod tests {
    use super::{compute_cover_crop, sample_source_coords_cover};

    #[test]
    fn cover_crop_crops_width_for_wide_source() {
        let crop = compute_cover_crop(1920, 1080, 80, 40, 0.5);
        assert!(crop.width < 1920);
        assert_eq!(crop.height, 1080);
    }

    #[test]
    fn cover_crop_crops_height_for_tall_source() {
        let crop = compute_cover_crop(800, 1200, 120, 30, 0.5);
        assert_eq!(crop.width, 800);
        assert!(crop.height < 1200);
    }

    #[test]
    fn sample_coords_cover_stay_within_bounds() {
        let (sx, sy) = sample_source_coords_cover(79, 23, 80, 24, 640, 480, 0.5);
        assert!(sx < 640);
        assert!(sy < 480);
    }
}
