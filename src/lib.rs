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
    Win32::Foundation::{RECT, COLORREF},
    Win32::UI::WindowsAndMessaging::{
        FindWindowA,
        GetClientRect,
        GetWindowLongA,
        SetWindowLongPtrA,
        GWL_EXSTYLE, // = WINDOW_LONG_PTR_INDEX(-20)
        SetLayeredWindowAttributes,
        LWA_ALPHA, // = LAYERED_WINDOW_ATTRIBUTES_FLAGS(2u32)
    },
    Win32::UI::Controls::MARGINS,
};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, ShowWindow, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE, SW_SHOW};

impl Overlay {
    pub fn new(font: &str, size:f32) -> Self {
        Self {
            window: HWND::default(),
            d2d_factory: None,
            target: None,
            write_factory: None,
            format: None,

            font: font.to_string(),
            font_size: size,
        }
    }

    // CORE FUNCTIONALITY ----------------
    /// Must be called prior to any rendering.
    pub fn init(&mut self) -> Result<(), OverlayError> {
        self.window = unsafe {
            // as_ptr() avoids allocating strings
            FindWindowA(
                PCSTR::from_raw("CEF-OSC-WIDGET\0".as_ptr()),
                PCSTR::from_raw("NVIDIA GeForce Overlay\0".as_ptr()),
            ).expect("Failed to find NVIDIA Overlay window.")
        };

        if self.window.is_invalid() {
            return Err(OverlayError::WindowNotFound);
        }

        let window_info = unsafe { GetWindowLongA(self.window, GWL_EXSTYLE) };
        if window_info == 0 {
            return Err(OverlayError::FailedToGetWindowLong);
        }

        let modify_window = unsafe {
            SetWindowLongPtrA(self.window, GWL_EXSTYLE, (window_info | 0x20) as isize)
        };
        if modify_window == 0 {
            return Err(OverlayError::FailedToSetWindowLong);
        }

        // Set window size equal to window
        let mut margins = MARGINS {
            cxLeftWidth: -1,   // Width of the left border
            cxRightWidth: -1,  // Width of the right border
            cyTopHeight: -1,   // Height of the top border
            cyBottomHeight: -1 // Height of the bottom border
        };

        let extend_frame_into_client_area = unsafe { DwmExtendFrameIntoClientArea(self.window, &mut margins) };
        if extend_frame_into_client_area.is_err() {
            return Err(OverlayError::FailedToExtendFrame);
        }

        let set_layered_window_attributes =
            unsafe { SetLayeredWindowAttributes(self.window, COLORREF(0x000000), 0xFF, LWA_ALPHA) };
        if set_layered_window_attributes.is_err() {
            return Err(OverlayError::FailedSetLayeredWindowAttributes);
        }
        
        let set_window_pos = unsafe {
            SetWindowPos(self.window, 
                         HWND_TOPMOST, 0,
                         0, 
                         0,
                         0, 
                         SWP_NOMOVE | SWP_NOSIZE
            )
        };
        if set_window_pos.is_err() {
            return Err(OverlayError::FailedToSetWindowPos);
        }

        // This is a windows BOOL so it's not just a !show_window comparison
        let show_window = unsafe {
            ShowWindow(self.window, SW_SHOW)
        };
        if !show_window.as_bool() {
            return Err(OverlayError::FailedToSetWindowPos);
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

        self.d2d_factory = Some(d2d_factory);
        self.write_factory = Some(write_factory);
        self.format = Some(format);
        self.target = Some(target);

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
}

impl Drop for Overlay {
    fn drop(&mut self) {
        self.begin_scene();
        self.clear_scene();
        self.end_scene();
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};
    use super::*;

    #[test]
    fn it_works() {
        let mut overlay = Overlay::new("Calibri", 18.0);

        // Initialize overlay
        let init = overlay.init();
        if init.is_err() {
            println!("init failed");
        }
        else {
            println!("init success");
        }

        // Startup overlay rendering
        let startup_d2d = overlay.startup_d2d();
        if startup_d2d.is_err() {
            println!("startup_d2d failed");
        }
        else {
            println!("startup_d2d success");
        }

        println!("Successfully initialized, rendering for 10 seconds now..\n");

        let red: (u8, u8, u8, u8) = (255, 51, 0, 255);
        let green: (u8, u8, u8, u8) = (0, 255, 51, 255);

        // Show the overlay for 10 seconds
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(10) {
            overlay.begin_scene();
            overlay.clear_scene();
            overlay.draw_text(
                (10.0, 30.0),
                "https://github.com/WakelandBranz/nvidia-overlay-hijack".to_string(),
                Some(red),
            ).expect("Failed to draw text");
            overlay.draw_rect((10.0, 80.0), (100.0, 100.0), 2.0, None).expect("Failed to draw rectangle");
            overlay.draw_circle((60.0, 250.0), 50.0, 4.0, Some(green)).expect("Failed to draw circle");
            overlay.end_scene();
        }

        println!("Done!");
    }
}