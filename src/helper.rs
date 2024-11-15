use std::ffi::OsStr;
use std::os::windows::prelude::OsStrExt;

use windows::{
    Win32::Foundation::{RECT, SUCCEEDED},
    Win32::Graphics::Direct2D::{
        ID2D1SolidColorBrush,
        ID2D1HwndRenderTarget,
        D2D1_BRUSH_PROPERTIES,
        D2D1_COLOR_F,
        D2D1_MATRIX_3X2_F,
        ID2D1Brush
    },
    Win32::Graphics::DirectWrite::IDWriteTextLayout,
    Win32::UI::WindowsAndMessaging::GetWindowRect,
};

use crate::Overlay;