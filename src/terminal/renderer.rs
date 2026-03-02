use crate::ascii::{
    color::{ColorMode, push_fg_escape},
    mapper::{AsciiCell, AsciiFrame},
};
use crossterm::terminal;
use std::io::{self, Stdout, Write};

pub struct TerminalRenderer {
    stdout: Stdout,
    prev_rows: Vec<String>,
    prev_width: u16,
    prev_height: u16,
}

impl TerminalRenderer {
    pub fn new() -> Self {
        Self {
            stdout: io::stdout(),
            prev_rows: Vec::new(),
            prev_width: 0,
            prev_height: 0,
        }
    }

    pub fn current_size(&self) -> io::Result<(u16, u16)> {
        terminal::size()
    }

    pub fn render(
        &mut self,
        frame: &AsciiFrame,
        color_mode: ColorMode,
        status_line: &str,
    ) -> io::Result<()> {
        let rows = build_rows(frame, color_mode);

        if self.prev_width != frame.width || self.prev_height != frame.height {
            write!(self.stdout, "\x1b[2J")?;
            self.prev_rows = vec![String::new(); frame.height as usize];
            self.prev_width = frame.width;
            self.prev_height = frame.height;
        }

        if self.prev_rows.len() != rows.len() {
            self.prev_rows.resize(rows.len(), String::new());
        }

        for (idx, row) in rows.iter().enumerate() {
            if self.prev_rows[idx] != *row {
                write!(self.stdout, "\x1b[{};1H{}\x1b[0m\x1b[K", idx + 1, row)?;
                self.prev_rows[idx] = row.clone();
            }
        }

        let status_row = frame.height as usize + 1;
        write!(self.stdout, "\x1b[{};1H{}\x1b[K", status_row, status_line)?;
        self.stdout.flush()
    }

    pub fn finish(&mut self) -> io::Result<()> {
        write!(self.stdout, "\x1b[0m")?;
        self.stdout.flush()
    }
}

fn build_rows(frame: &AsciiFrame, color_mode: ColorMode) -> Vec<String> {
    let mut rows = Vec::with_capacity(frame.height as usize);
    for y in 0..frame.height {
        rows.push(build_row(frame.row(y), color_mode));
    }
    rows
}

fn build_row(cells: &[AsciiCell], color_mode: ColorMode) -> String {
    if color_mode == ColorMode::None {
        let mut out = String::with_capacity(cells.len());
        for cell in cells {
            out.push(cell.glyph as char);
        }
        return out;
    }

    let mut out = String::with_capacity(cells.len() * 8);
    let mut last_color: Option<(u8, u8, u8)> = None;

    for cell in cells {
        let color = (cell.r, cell.g, cell.b);
        if last_color != Some(color) {
            push_fg_escape(&mut out, color_mode, cell.r, cell.g, cell.b);
            last_color = Some(color);
        }
        out.push(cell.glyph as char);
    }

    out.push_str("\x1b[0m");
    out
}
