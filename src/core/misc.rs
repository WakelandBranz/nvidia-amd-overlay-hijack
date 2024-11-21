use crate::core::{Overlay, OverlayError};

impl Overlay {
    pub fn get_text_width(&self, text: impl ToString) -> Result<i32, OverlayError> {
        let text_length = text.to_string().len() as i32;
        match self.font_width {
            Some(width) => Ok(text_length * width),
            None => Err(OverlayError::FailedToGetFontWidth)
        }
    }
}