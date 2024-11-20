use std::ffi::OsStr;
use std::os::windows::prelude::OsStrExt;

use windows::{
    core::{Interface, Result as WindowsResult}, // Use com pointers to use idiomatic rust error handling
    Foundation::Numerics::Matrix3x2,
    Win32::Foundation::RECT,
    Win32::Graphics::Direct2D::{
        ID2D1SolidColorBrush,
        ID2D1HwndRenderTarget,
        D2D1_BRUSH_PROPERTIES,
        ID2D1Brush,
        Common::D2D1_COLOR_F
    },
    Win32::Graphics::DirectWrite::IDWriteTextLayout,
    Win32::UI::WindowsAndMessaging::GetWindowRect,
};
use crate::Overlay;


pub trait OverlayHelper {
    fn create_brush(&self, color: (u8, u8, u8, u8)) -> WindowsResult<ID2D1SolidColorBrush>;
    fn create_text_layout(&self, text: &str) -> WindowsResult<IDWriteTextLayout>;

    // Match the implementation signature
    fn draw_element<F>(&self, color: (u8, u8, u8, u8), draw: F) -> WindowsResult<()>
    where
        F: Fn(&ID2D1HwndRenderTarget, &ID2D1Brush);
}

impl OverlayHelper for Overlay {
    fn create_brush(&self, (r, g, b, a): (u8, u8, u8, u8)) -> WindowsResult<ID2D1SolidColorBrush> {
        let target = self.target.as_ref()
            .expect("No render target available");
        let brush_properties = create_brush_properties();
        let color = color_u8_to_f32((r, g, b, a));

        unsafe {
            // CreateSolidColorBrush returns Result<ID2D1SolidColorBrush>
            target.CreateSolidColorBrush(&color, Some(&brush_properties))
        }
    }

    fn create_text_layout(&self, text: &str) -> WindowsResult<IDWriteTextLayout> {
        let format = self.format.as_ref()
            .expect("No text format available");

        let mut rc: RECT = RECT::default();
        unsafe {
            GetWindowRect(self.window, &mut rc)
                .expect("GetWindowRect failed in overlay::helper::create_text_layout");
        }

        let text_wide: Vec<u16> = OsStr::new(text)
            .encode_wide()
            .chain(Some(0).into_iter())
            .collect();

        unsafe {
            self.write_factory
                .as_ref()
                .unwrap()
                .CreateTextLayout(
                    &text_wide,
                    self.format.as_ref().unwrap(),
                    (rc.right - rc.left) as f32,
                    (rc.bottom - rc.top) as f32,
                )
        }
    }

    fn draw_element<F>(&self, color: (u8, u8, u8, u8), draw: F) -> WindowsResult<()>
    where
        F: Fn(&ID2D1HwndRenderTarget, &ID2D1Brush)
    {
        let brush = self.create_brush(color)?;
        let target = self.target.as_ref()
            .expect("No render target available");

        // Cast the brush
        let brush_interface: ID2D1Brush = unsafe { brush.cast()? };

        draw(target, &brush_interface);

        Ok(())
    }
}

// Auxiliary functions
fn create_brush_properties() -> D2D1_BRUSH_PROPERTIES {
    D2D1_BRUSH_PROPERTIES {
        opacity: 1.0f32,
        transform: Matrix3x2 {
            M11: 1.0,  // Scale X
            M12: 0.0,  // Rotation/Skew
            M21: 0.0,  // Rotation/Skew
            M22: 1.0,  // Scale Y
            M31: 0.0,  // Translation X
            M32: 0.0,  // Translation Y
        },
    }
}

fn color_u8_to_f32((r, g, b, a): (u8, u8, u8, u8)) -> D2D1_COLOR_F {
    D2D1_COLOR_F {
        r: r as f32 / 255.0f32,
        g: g as f32 / 255.0f32,
        b: b as f32 / 255.0f32,
        a: a as f32 / 255.0f32,
    }
}