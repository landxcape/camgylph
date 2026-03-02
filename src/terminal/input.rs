use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::{io, time::Duration};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Control {
    Quit,
    ToggleColorMode,
    ToggleRamp,
    ToggleMetrics,
    IncreaseGamma,
    DecreaseGamma,
    IncreaseContrast,
    DecreaseContrast,
}

pub fn poll_controls() -> io::Result<Vec<Control>> {
    let mut controls = Vec::new();

    while event::poll(Duration::from_millis(0))? {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press && key.kind != KeyEventKind::Repeat {
                continue;
            }

            match key.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                    controls.push(Control::Quit)
                }
                KeyCode::Char('c') | KeyCode::Char('C') => controls.push(Control::ToggleColorMode),
                KeyCode::Char('r') | KeyCode::Char('R') => controls.push(Control::ToggleRamp),
                KeyCode::Char('m') | KeyCode::Char('M') => controls.push(Control::ToggleMetrics),
                KeyCode::Char('=') | KeyCode::Char('+') => controls.push(Control::IncreaseGamma),
                KeyCode::Char('-') => controls.push(Control::DecreaseGamma),
                KeyCode::Char(']') => controls.push(Control::IncreaseContrast),
                KeyCode::Char('[') => controls.push(Control::DecreaseContrast),
                _ => {}
            }
        }
    }

    Ok(controls)
}
