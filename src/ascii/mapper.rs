use crate::frame::{
    frame::RgbFrameView, luminance::rgb_luma_u8, resize::sample_source_coords_cover,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AsciiCell {
    pub glyph: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub struct AsciiFrame {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<AsciiCell>,
}

impl AsciiFrame {
    pub fn row(&self, y: u16) -> &[AsciiCell] {
        let start = y as usize * self.width as usize;
        let end = start + self.width as usize;
        &self.cells[start..end]
    }
}

pub fn map_rgb_frame(
    frame: &RgbFrameView<'_>,
    out_width: u16,
    out_height: u16,
    ramp: &[u8],
    gamma: f32,
    contrast: f32,
    cell_aspect_ratio: f32,
    mirror_horizontal: bool,
) -> AsciiFrame {
    let width = out_width.max(1);
    let height = out_height.max(1);
    let mut cells = Vec::with_capacity(width as usize * height as usize);

    for y in 0..height {
        for x in 0..width {
            let sample_x = if mirror_horizontal { width - 1 - x } else { x };
            let (sx, sy) = sample_source_coords_cover(
                sample_x,
                y,
                width,
                height,
                frame.width,
                frame.height,
                cell_aspect_ratio,
            );
            let (r, g, b) = frame.pixel_at(sx, sy);
            let luma = tone_adjust(rgb_luma_u8(r, g, b), gamma, contrast);
            let glyph = luma_to_glyph(luma, ramp);
            cells.push(AsciiCell { glyph, r, g, b });
        }
    }

    AsciiFrame {
        width,
        height,
        cells,
    }
}

#[inline]
fn luma_to_glyph(luma: u8, ramp: &[u8]) -> u8 {
    if ramp.is_empty() {
        return b' ';
    }

    let idx = (luma as usize * (ramp.len() - 1)) / 255;
    ramp[idx]
}

#[inline]
fn tone_adjust(luma: u8, gamma: f32, contrast: f32) -> u8 {
    let gamma = gamma.clamp(0.1, 4.0);
    let contrast = contrast.clamp(0.2, 3.0);

    let mut v = luma as f32 / 255.0;
    v = v.powf(1.0 / gamma);
    v = ((v - 0.5) * contrast + 0.5).clamp(0.0, 1.0);
    (v * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::map_rgb_frame;
    use crate::frame::frame::RgbFrameView;

    #[test]
    fn maps_tiny_frame() {
        let data = vec![
            0, 0, 0, 255, 255, 255, //
            255, 0, 0, 0, 255, 0, //
        ];
        let frame = RgbFrameView::new(2, 2, &data).expect("valid frame");
        let mapped = map_rgb_frame(&frame, 2, 2, b" .#", 1.0, 1.0, 0.5, false);
        assert_eq!(mapped.width, 2);
        assert_eq!(mapped.height, 2);
        assert_eq!(mapped.cells.len(), 4);
    }

    #[test]
    fn mirrors_frame_horizontally_when_enabled() {
        let data = vec![
            0, 0, 0, //
            255, 255, 255,
        ];
        let frame = RgbFrameView::new(2, 1, &data).expect("valid frame");
        let normal = map_rgb_frame(&frame, 2, 1, b" .#", 1.0, 1.0, 1.0, false);
        let mirrored = map_rgb_frame(&frame, 2, 1, b" .#", 1.0, 1.0, 1.0, true);

        assert_eq!(normal.cells[0].glyph, b' ');
        assert_eq!(normal.cells[1].glyph, b'#');
        assert_eq!(mirrored.cells[0].glyph, b'#');
        assert_eq!(mirrored.cells[1].glyph, b' ');
    }
}
