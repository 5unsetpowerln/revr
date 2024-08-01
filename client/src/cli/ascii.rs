use std::char;

use anyhow::{bail, Result};

/// converts caret notation to corresponding control character
///
/// lowercase alphabets don't have any corresponding control characters
///
/// https://en.wikipedia.org/wiki/ASCII#Control_code_chart
pub fn char_to_ctrl(character: u8) -> Result<u8> {
    if character > 0x5f || character < 0x3f {
        bail!("this character doesn't have any corresponding control character");
    }
    match character {
        0x3f => return Ok(0x7f),
        _ => return Ok(character - 0x40),
    }
}

/// converts control character to corresponding caret notation
pub fn ctrl_to_char(ctrl_character: u8) -> Result<u8> {
    if ctrl_character > 0x1f || ctrl_character != 0x7f {
        bail!("this control character doesn't have any corresponding character");
    }
    match ctrl_character {
        0x7f => return Ok(0x3f),
        _ => return Ok(ctrl_character + 0x40),
    }
}
