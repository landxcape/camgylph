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
    prev_status_row: Option<usize>,
}

impl TerminalRenderer {
    pub fn new() -> Self {
        Self {
            stdout: io::stdout(),
            prev_rows: Vec::new(),
            prev_width: 0,
            prev_height: 0,
            prev_status_row: None,
        }
    }

    pub fn current_size(&self) -> io::Result<(u16, u16)> {
        terminal::size()
    }

    pub fn render(
        &mut self,
        frame: &AsciiFrame,
        color_mode: ColorMode,
        status_line: Option<&str>,
    ) -> io::Result<()> {
        let rows = build_rows(frame, color_mode);
        let (term_w, _) = terminal::size()?;

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

        if let Some(status_line) = status_line {
            let status_row = frame.height as usize + 1;
            let clamped = clamp_to_columns(status_line, term_w as usize);
            write!(self.stdout, "\x1b[{};1H{}\x1b[K", status_row, clamped)?;
            self.prev_status_row = Some(status_row);
        } else if let Some(prev_status_row) = self.prev_status_row.take() {
            write!(self.stdout, "\x1b[{};1H\x1b[K", prev_status_row)?;
        }

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

fn clamp_to_columns(text: &str, max_cols: usize) -> String {
    if max_cols == 0 {
        return String::new();
    }

    text.chars().take(max_cols).collect()
}
