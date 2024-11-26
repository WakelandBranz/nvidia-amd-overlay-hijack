// Heavily *inspired* by https://github.com/WilgnerFSDev/nvidia-overlay-hijack-rs/tree/main
// Thank you WilgnerFSDev
pub mod core;
pub mod helper;

use crate::{
    core::{Overlay, OverlayError},
};

use std::ffi::OsStr;
use std::os::windows::prelude::OsStrExt;
use windows::{
    core::{PCSTR,
           PCWSTR,
           w, // A literal UTF-16 wide string with a trailing null terminator.
    },
    Win32::Graphics::{
        Direct2D::{
            D2D1CreateFactory,
            ID2D1Factory,
            D2D1_FACTORY_TYPE_SINGLE_THREADED,
            D2D1_FEATURE_LEVEL_DEFAULT,
            D2D1_HWND_RENDER_TARGET_PROPERTIES,
            D2D1_PRESENT_OPTIONS_NONE,
            D2D1_RENDER_TARGET_PROPERTIES,
            D2D1_RENDER_TARGET_TYPE_DEFAULT,
            D2D1_RENDER_TARGET_USAGE_NONE,
            Common::{
                D2D_SIZE_U,
                D2D1_ALPHA_MODE_PREMULTIPLIED,
                D2D1_PIXEL_FORMAT,
            },
        },
        DirectWrite::{
            DWriteCreateFactory,
            IDWriteFactory,
            IDWriteTextFormat,
            DWRITE_FACTORY_TYPE_SHARED,
            DWRITE_FONT_STRETCH_NORMAL,
            DWRITE_FONT_STYLE_NORMAL,
            DWRITE_FONT_WEIGHT_REGULAR,
        },
        Dxgi::Common::DXGI_FORMAT_UNKNOWN,
        Dwm::DwmExtendFrameIntoClientArea,
    },
    Win32::Foundation::{RECT, COLORREF, HWND},
    Win32::UI::WindowsAndMessaging::{
        FindWindowA,
        GetClientRect,
        GetWindowLongA,
        SetWindowLongPtrA,
        GWL_EXSTYLE, // = WINDOW_LONG_PTR_INDEX(-20)
        SetLayeredWindowAttributes,
        LWA_ALPHA, // = LAYERED_WINDOW_ATTRIBUTES_FLAGS(2u32)
        SetWindowPos,
        //ShowWindow,
        HWND_TOPMOST,
        SWP_NOMOVE,
        SWP_NOSIZE,
        //SW_SHOW,
    },
    Win32::UI::Controls::MARGINS,
};

const LAYERED_WINDOW_STYLE: i32 = 0x20;
const WINDOW_ALPHA: u8 = 0xFF;

impl Overlay {
    pub fn new(font: impl ToString, size:f32) -> Self {
        Self {
            window: HWND::default(),
            d2d_factory: None, // Used for rendering 2d objects
            target: None, // Used for rendering 2d objects
            write_factory: None,
            format: None,

            font: font.to_string(),
            font_size: size,
            font_width: None, // This will be useful for calculating the width of a rendered string
        }
    }

    // CORE FUNCTIONALITY ----------------
    /// Must be called prior to any rendering.
    pub fn init(&mut self) -> Result<(), OverlayError> {
        // Find and validate window
        self.window = unsafe {
            FindWindowA(
                PCSTR::from_raw("CEF-OSC-WIDGET\0".as_ptr()),
                PCSTR::from_raw("NVIDIA GeForce Overlay\0".as_ptr()),
            ).map_err(|_| OverlayError::WindowNotFound)?
        };

        // Set window style
        let window_info = unsafe { GetWindowLongA(self.window, GWL_EXSTYLE) };
        if window_info == 0 {
            return Err(OverlayError::FailedToGetWindowLong);
        }

        let modified_style = window_info | LAYERED_WINDOW_STYLE;
        let modify_window = unsafe {
            SetWindowLongPtrA(self.window, GWL_EXSTYLE, modified_style as isize)
        };
        if modify_window == 0 {
            return Err(OverlayError::FailedToSetWindowLong);
        }

        // Configure window margins
        let margins = MARGINS {
            cxLeftWidth: -1,
            cxRightWidth: -1,
            cyTopHeight: -1,
            cyBottomHeight: -1,
        };

        // Set window properties
        unsafe {
            DwmExtendFrameIntoClientArea(self.window, &margins)
                .map_err(|_| OverlayError::FailedToExtendFrame)?;

            SetLayeredWindowAttributes(
                self.window,
                COLORREF(0x000000),
                WINDOW_ALPHA,
                LWA_ALPHA
            ).map_err(|_| OverlayError::FailedSetLayeredWindowAttributes)?;

            SetWindowPos(
                self.window,
                HWND_TOPMOST,
                0, 0, 0, 0,
                SWP_NOMOVE | SWP_NOSIZE
            ).map_err(|_| OverlayError::FailedToSetWindowPos)?;

            //if !ShowWindow(self.window, SW_SHOW).as_bool() {
            //    return Err(OverlayError::FailedToShowWindow); // More specific error
            //}
        }

        Ok(())
    }

    pub fn startup_d2d(&mut self) -> Result<(), OverlayError> {
        let d2d_factory: ID2D1Factory = unsafe {
            match D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None) {
                Ok(factory) => factory,
                Err(_) => return Err(OverlayError::ID2D1FactoryFailed),
            }
        };

        let write_factory: IDWriteFactory = unsafe {
            match DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED) {
                Ok(factory) => factory,
                Err(_) => return Err(OverlayError::IDWriteFactoryFailed)
            }
        };

        let font_wide: Vec<u16> = OsStr::new(&self.font)
            .encode_wide()
            .chain(Some(0).into_iter())
            .collect();

        let format: IDWriteTextFormat = unsafe {
            match write_factory.CreateTextFormat(
                PCWSTR::from_raw(font_wide.as_ptr()),
                None,
                DWRITE_FONT_WEIGHT_REGULAR,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                self.font_size,
                w!("en-us"),
            )
            {
                Ok(format) => format,
                Err(_) => return Err(OverlayError::IDWriteTextFormatFailed),
            }
        };

        let mut rect = RECT::default();
        if let Err(_) = unsafe { GetClientRect(self.window, &mut rect) } {
            return Err(OverlayError::GetWindowRectFailed);
        }

        let render_target_properties = D2D1_RENDER_TARGET_PROPERTIES {
            r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_UNKNOWN,
                alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
            },
            dpiX: 0.0,
            dpiY: 0.0,
            usage: D2D1_RENDER_TARGET_USAGE_NONE,
            minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
        };

        let hwnd_target_properties = D2D1_HWND_RENDER_TARGET_PROPERTIES {
            hwnd: self.window,
            pixelSize: D2D_SIZE_U {
                width: (rect.right - rect.left) as u32,
                height: (rect.bottom - rect.top) as u32,
            },
            presentOptions: D2D1_PRESENT_OPTIONS_NONE,
        };

        let target = unsafe {
            match d2d_factory.CreateHwndRenderTarget(&render_target_properties, &hwnd_target_properties) {
                Ok(target) => target,
                Err(_) => return Err(OverlayError::StartupD2DFailed),
            }
        };

        let font_width: i32 = unsafe {
            format.GetFontStretch().0
        };

        self.d2d_factory = Some(d2d_factory);
        self.write_factory = Some(write_factory);
        self.format = Some(format);
        self.target = Some(target);
        self.font_width = Some(font_width);

        Ok(())
    }

    // We want a reference to the value inside the option, so we use .as_ref() to get Option<&T>
    pub fn begin_scene(&mut self) {
        match self.target.as_ref() {
            Some(target) => unsafe { target.BeginDraw() },
            None => panic!("Render Target is None -> Attempted begin_scene without initializing overlay!"),
        }
    }

    pub fn end_scene(&mut self) {
        match self.target.as_ref() {
            Some(target) => unsafe { target.EndDraw(None, None).expect("Failed to end scene.") },
            None => panic!("Render Target is None -> Attempted begin_scene without initializing overlay!"),
        }
    }

    pub fn clear_scene(&mut self) {
        match self.target.as_ref() {
            Some(target) => unsafe { target.Clear(None) },
            None => panic!("Render Target is None -> Attempted clear_scene without initializing overlay!"),
        }
    }

    /// Begins, clears, and ends the scene. Saves you on some lines of code
    pub fn force_clear_scene(&mut self) {
        self.begin_scene();
        self.clear_scene();
        self.end_scene();
    }

    pub fn cleanup(&mut self) {
        if self.target.is_some() {
            self.force_clear_scene();

            // Explicitly drop D2D resources in the correct order
            self.format.take();  // Drop text format first
            self.target.take();  // Drop render target next
            self.write_factory.take();  // Drop factories last
            self.d2d_factory.take();
        }
    }
}

impl Drop for Overlay {
    fn drop(&mut self) {
        self.cleanup()
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};
    use windows::Win32::Graphics::Direct2D::D2D1_CAP_STYLE_ROUND;
    use super::*;

    #[test]
    fn it_works() {
        let mut overlay = Overlay::new("Comic Sans MS", 18.0);

        // Initialize nvidia_overlay
        match overlay.init() {
            Ok(_) => println!("Successfully initialized overlay"),
            Err(_) => println!("Failed to initialize overlay")
        };

        // Startup nvidia_overlay rendering
         match overlay.startup_d2d() {
             Ok(_) => println!("Succeeded in startup_d2d"),
             Err(_) => println!("Failed startup_d2d"),
         };

        println!("Successfully initialized, rendering for 10 seconds now..\n");

        let red = (255, 51, 0, 255);
        let green = (0, 255, 51, 255);
        let blue = (0, 51, 255, 255);
        let yellow = (255, 255, 0, 255);
        let purple = (255, 0, 255, 255);
        let cyan = (0, 255, 255, 255);

        // Show the nvidia_overlay for 10 seconds
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(10) {
            overlay.begin_scene();
            overlay.clear_scene();

            // Text at the top
            overlay.draw_text(
                (10.0, 30.0),
                "https://github.com/WakelandBranz/nvidia-overlay-hijack\nShape Showcase".to_string(),
                red,
            ).expect("Failed to draw text");

            // Basic shapes
            overlay.draw_rect(
                (10.0, 100.0),
                (100.0, 80.0),
                2.0,
                yellow
            ).expect("Failed to draw rectangle");

            overlay.draw_filled_rect(
                (120.0, 100.0),
                (100.0, 80.0),
                green
            ).expect("Failed to draw filled rectangle");

            overlay.draw_gradient_rect(
                (230.0, 100.0),
                (100.0, 80.0),
                red,
                blue,
                true
            ).expect("Failed to draw gradient rectangle");

            // Rounded rectangles
            overlay.draw_rounded_rect(
                (10.0, 200.0),
                (100.0, 80.0),
                10.0,
                2.0,
                purple
            ).expect("Failed to draw rounded rectangle");

            overlay.draw_filled_rounded_rect(
                (120.0, 200.0),
                (100.0, 80.0),
                10.0,
                cyan
            ).expect("Failed to draw filled rounded rectangle");

            overlay.draw_gradient_rounded_rect(
                (230.0, 200.0),
                (100.0, 80.0),
                10.0,
                green,
                purple,
                false
            ).expect("Failed to draw gradient rounded rectangle");

            // Circles and Ellipses
            overlay.draw_circle(
                (60.0, 350.0),
                30.0,
                2.0,
                yellow
            ).expect("Failed to draw circle");

            overlay.draw_filled_circle(
                (170.0, 350.0),
                30.0,
                blue
            ).expect("Failed to draw filled circle");

            overlay.draw_gradient_circle(
                (280.0, 350.0),
                30.0,
                red,
                blue,
                true
            ).expect("Failed to draw gradient circle (radial)");

            // Ellipses
            overlay.draw_ellipse(
                (60.0, 450.0),
                (40.0, 25.0),
                2.0,
                green
            ).expect("Failed to draw ellipse");

            overlay.draw_filled_ellipse(
                (170.0, 450.0),
                (40.0, 25.0),
                purple
            ).expect("Failed to draw filled ellipse");

            overlay.draw_gradient_ellipse(
                (280.0, 450.0),
                (40.0, 25.0),
                yellow,
                cyan,
                false
            ).expect("Failed to draw gradient ellipse (linear)");

            // Regular line
            overlay.draw_line(
                (400.0, 100.0),
                (500.0, 150.0),
                2.0,
                yellow
            ).expect("Failed to draw line");

            // Gradient line
            overlay.draw_gradient_line(
                (400.0, 200.0),
                (500.0, 250.0),
                3.0,
                red,
                blue
            ).expect("Failed to draw gradient line");

            // Dashed line
            overlay.draw_styled_line(
                (400.0, 300.0),
                (500.0, 350.0),
                2.0,
                green,
                D2D1_CAP_STYLE_ROUND,
                D2D1_CAP_STYLE_ROUND
            ).expect("Failed to draw styled line");

            overlay.end_scene();
        }

        println!("Done!");
    }
}