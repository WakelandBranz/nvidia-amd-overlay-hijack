mod draw;
mod misc;

use std::collections::HashMap;
use windows::{
    Win32::{
        Foundation::{
            HWND,
            RECT,
        },
        Graphics::{
            Direct2D::{
                ID2D1Factory,
                ID2D1HwndRenderTarget,
                ID2D1LinearGradientBrush,
                ID2D1RadialGradientBrush,
                ID2D1SolidColorBrush,
            },
            DirectWrite::{
                IDWriteFactory,
                IDWriteTextFormat,
                IDWriteTextLayout,
            }
        }
    },
};

// SAFETY: HWND is thread-safe as it's just an identifier
unsafe impl Send for Overlay {}
unsafe impl Sync for Overlay {}

pub struct Overlay {
    pub window: HWND,
    pub d2d_factory: Option<ID2D1Factory>,
    pub target: Option<ID2D1HwndRenderTarget>,
    pub write_factory: Option<IDWriteFactory>,
    pub format: Option<IDWriteTextFormat>,

    // Font specific rendering fields
    pub font: String,
    pub font_size: f32,
    pub font_width: Option<i32>,

    // Caches for performance
    pub solid_color_brush: Option<ID2D1SolidColorBrush>,
    pub linear_gradient_brush: Option<ID2D1LinearGradientBrush>,
    pub radial_gradient_brush: Option<ID2D1RadialGradientBrush>,
    pub window_size: Option<RECT>,
    pub text_layout_cache: HashMap<String, IDWriteTextLayout>, // IDWriteTextLayout is just a COM Pointer
    pub cache_frame_count: u32, // Track frames for periodic cleanup
}

#[derive(Debug)]
pub enum OverlayError {
    WindowNotFound,
    FailedToGetWindowLong,
    FailedToSetWindowLong,
    FailedToExtendFrame,
    FailedSetLayeredWindowAttributes,
    FailedToSetWindowPos,
    ShowWindowFailed,

    ID2D1FactoryFailed,
    StartupD2DFailed,
    IDWriteFactoryFailed,
    IDWriteTextFormatFailed,

    NoRenderTarget,
    GetWindowRectFailed,
    GetWriteTextFormatFailed,
    DrawFailed,
    DrawTextFailed(i32),
    FailedToGetFontWidth,
    CreateBrushFailed(i32),
    CreateSolidColorBrushFailed,
    ID2D1BrushCastFailed,
    CreateGradientStopCollectionFailed,
    CreateLinearGradientBrushFailed,
    CreateRadialGradientBrushFailed,
    NoD2DFactory,
    CreateStrokeStyleFailed,
    FailedToShowWindow,
}