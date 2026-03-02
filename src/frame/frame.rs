#[derive(Clone, Copy)]
pub struct RgbFrameView<'a> {
    pub width: u32,
    pub height: u32,
    pub data: &'a [u8],
}

impl<'a> RgbFrameView<'a> {
    pub fn new(width: u32, height: u32, data: &'a [u8]) -> Option<Self> {
        let expected = (width as usize)
            .saturating_mul(height as usize)
            .saturating_mul(3);
        if data.len() == expected {
            Some(Self {
                width,
                height,
                data,
            })
        } else {
            None
        }
    }

    pub fn pixel_at(&self, x: u32, y: u32) -> (u8, u8, u8) {
        let idx = ((y * self.width + x) * 3) as usize;
        (self.data[idx], self.data[idx + 1], self.data[idx + 2])
    }
}
