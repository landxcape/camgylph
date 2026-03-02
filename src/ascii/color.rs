#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorMode {
    Truecolor,
    Ansi256,
    None,
}

impl ColorMode {
    pub fn next(self) -> Self {
        match self {
            Self::Truecolor => Self::Ansi256,
            Self::Ansi256 => Self::None,
            Self::None => Self::Truecolor,
        }
    }
}

pub fn mode_label(mode: ColorMode) -> &'static str {
    match mode {
        ColorMode::Truecolor => "truecolor",
        ColorMode::Ansi256 => "ansi256",
        ColorMode::None => "none",
    }
}

pub fn push_fg_escape(out: &mut String, mode: ColorMode, r: u8, g: u8, b: u8) {
    match mode {
        ColorMode::Truecolor => {
            out.push_str("\x1b[38;2;");
            out.push_str(&r.to_string());
            out.push(';');
            out.push_str(&g.to_string());
            out.push(';');
            out.push_str(&b.to_string());
            out.push('m');
        }
        ColorMode::Ansi256 => {
            let idx = rgb_to_ansi256(r, g, b);
            out.push_str("\x1b[38;5;");
            out.push_str(&idx.to_string());
            out.push('m');
        }
        ColorMode::None => {}
    }
}

fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    if r == g && g == b {
        if r < 8 {
            return 16;
        }
        if r > 248 {
            return 231;
        }
        return 232 + ((r as u16 - 8) / 10) as u8;
    }

    let r6 = (r as u16 * 5 / 255) as u8;
    let g6 = (g as u16 * 5 / 255) as u8;
    let b6 = (b as u16 * 5 / 255) as u8;
    16 + 36 * r6 + 6 * g6 + b6
}
