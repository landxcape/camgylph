use std::io;

pub fn resolve_input_spec(device: Option<&str>) -> io::Result<String> {
    #[cfg(target_os = "macos")]
    {
        return Ok(device.unwrap_or("0:none").to_string());
    }

    #[cfg(target_os = "linux")]
    {
        return Ok(device.unwrap_or("/dev/video0").to_string());
    }

    #[cfg(target_os = "windows")]
    {
        let raw = device.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Windows needs --device \"Integrated Camera\" (or CAMGLYPH_DEVICE)",
            )
        })?;

        if raw.starts_with("video=") {
            return Ok(raw.to_string());
        }

        return Ok(format!("video={raw}"));
    }

    #[allow(unreachable_code)]
    Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported OS"))
}

#[cfg(target_os = "linux")]
pub fn listing_target(device: Option<&str>) -> String {
    device.unwrap_or("/dev/video0").to_string()
}
