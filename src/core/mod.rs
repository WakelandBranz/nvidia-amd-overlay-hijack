mod draw;
mod misc;

use windows::{
    Win32::Foundation::HWND,
    Win32::Graphics::{
        Direct2D::{ID2D1Factory, ID2D1HwndRenderTarget},
        DirectWrite::{IDWriteFactory, IDWriteTextFormat},
    },
};

// SAFETY: HWND is thread-safe as it's just an identifier
// Not sure why this is necessary though.
unsafe impl Send for Overlay {}
unsafe impl Sync for Overlay {}

pub struct Overlay {
    pub window: HWND,
    pub d2d_factory: Option<ID2D1Factory>,
    pub target: Option<ID2D1HwndRenderTarget>,
    pub write_factory: Option<IDWriteFactory>,
    pub format: Option<IDWriteTextFormat>,
    // ... other fields...
    pub font: String,
    pub font_size: f32,
    pub font_width: Option<i32>,
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
}