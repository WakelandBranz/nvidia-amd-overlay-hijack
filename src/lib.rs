// Heavily *inspired* by https://github.com/WilgnerFSDev/nvidia-overlay-hijack-rs/tree/main
// Thank you WilgnerFSDev
pub mod core;
pub mod helper;

use crate::{
    core::{Overlay, OverlayError},
    helper::{OverlayHelper},
};

use std::ffi::OsStr;
use std::os::windows::prelude::OsStrExt;
use std::ptr::null_mut;
// Figure out imports
use windows::{
    core::{Interface, PCSTR, PCWSTR, Result as WindowsResult},
    Win32::Graphics::{
        Direct2D::{
            D2D1CreateFactory,
            ID2D1Factory,
            ID2D1HwndRenderTarget,
            D2D1_FACTORY_TYPE_SINGLE_THREADED,
            D2D1_FEATURE_LEVEL_DEFAULT,
            D2D1_HWND_RENDER_TARGET_PROPERTIES,
            D2D1_POINT_DESCRIPTION,
            D2D1_PRESENT_OPTIONS_NONE,
            D2D1_RENDER_TARGET_PROPERTIES,
            D2D1_RENDER_TARGET_TYPE_DEFAULT,
            D2D1_RENDER_TARGET_USAGE_NONE,
            D2D1_DRAW_TEXT_OPTIONS_NONE,
            Common::{
                D2D_RECT_F,
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
    Win32::Foundation::{RECT, BOOL, SUCCESS, COLORREF},
    Win32::UI::WindowsAndMessaging::{
        FindWindowA,
        GetClientRect,
        GetWindowLongA,
        SetWindowLongPtrA,
        GWL_EXSTYLE, // = WINDOW_LONG_PTR_INDEX(-20)
        SetLayeredWindowAttributes,
        LWA_ALPHA, // = LAYERED_WINDOW_ATTRIBUTES_FLAGS(2u32)
        LAYERED_WINDOW_ATTRIBUTES_FLAGS
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

    // Core functionality ----------------
    pub fn init(&mut self) -> Result<(), OverlayError::WindowNotFound> {
        self.window = unsafe {
            // as_ptr() avoids allocating strings
            FindWindowA(
                PCSTR::from_raw("CEF-OSC-WIDGET\0".as_ptr()),
                PCSTR::from_raw("NVIDIA GeForce Overlay\0".as_ptr()),
            )
        }?;

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
        
        let show_window = unsafe {
            ShowWindow(self.window, SW_SHOW)
        };
        if !show_window {
            return Err(OverlayError::FailedToSetWindowPos);
        }

        Ok(())
    }
}