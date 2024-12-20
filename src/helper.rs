use std::ffi::OsStr;
use std::os::windows::prelude::OsStrExt;
use str_crypter::{decrypt_string, sc};
use windows::{
    core::{
        Interface,
        Result as WindowsResult,
        PCSTR,
    },
    Win32::{
        Foundation::{
            HWND,
            RECT,
        },
        Graphics::{
            Direct2D::{
                ID2D1SolidColorBrush,
                ID2D1HwndRenderTarget,
                D2D1_BRUSH_PROPERTIES,
                ID2D1Brush,
                Common::D2D1_COLOR_F,
            },
            DirectWrite::IDWriteTextLayout,
        },
        UI::WindowsAndMessaging::{
            GetWindowRect,
            FindWindowA,
        }
    },
    Foundation::{
        Numerics::{
            Matrix3x2,
        }
    },
};
use windows::Win32::Graphics::Direct2D::Common::{D2D1_GRADIENT_STOP, D2D_POINT_2F};
use windows::Win32::Graphics::Direct2D::{ID2D1LinearGradientBrush, ID2D1RadialGradientBrush, D2D1_EXTEND_MODE_CLAMP, D2D1_GAMMA_2_2, D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES, D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES};
use crate::core::OverlayError;
use crate::Overlay;


pub trait OverlayHelper {
    fn draw_with_brush<F, T: Interface>(&self, brush: &T, draw: F) -> WindowsResult<()>
    where
        F: Fn(&ID2D1HwndRenderTarget, &ID2D1Brush),
        T: Interface;
    fn create_text_layout(&self, text: &str) -> WindowsResult<IDWriteTextLayout>;
    fn update_text_layout(&mut self, text: &str) -> Result<IDWriteTextLayout, OverlayError>;
    /// Call this in your render loop
    fn try_clear_text_layout_cache(&mut self);
    fn create_solid_color_brush(&self, color: (u8, u8, u8, u8)) -> WindowsResult<ID2D1SolidColorBrush>;
    fn update_solid_color_brush(
        &mut self,
        color: (u8, u8, u8, u8),
    ) -> Result<(), OverlayError>;
    fn create_linear_gradient_brush(
        &self,
        start_point: (f32, f32),
        end_point: (f32, f32),
        color1: (u8, u8, u8, u8),
        color2: (u8, u8, u8, u8),
    ) -> WindowsResult<ID2D1LinearGradientBrush>;
    fn update_linear_gradient_brush(
        &mut self,
        start_point: (f32, f32),
        end_point: (f32, f32),
        color1: (u8, u8, u8, u8),
        color2: (u8, u8, u8, u8),
    ) -> Result<(), OverlayError>;

    fn create_radial_gradient_brush(
        &self,
        center: (f32, f32),
        radius: (f32, f32),
        color1: (u8, u8, u8, u8),
        color2: (u8, u8, u8, u8),
    ) -> WindowsResult<ID2D1RadialGradientBrush>;
    /// Compares gradient stops to see if they need to be updated, otherwise just modify start and end.
    fn update_radial_gradient_brush(
        &mut self,
        start_point: (f32, f32),
        end_point: (f32, f32),
        color1: (u8, u8, u8, u8),
        color2: (u8, u8, u8, u8),
    ) -> Result<(), OverlayError>;
}

impl OverlayHelper for Overlay {
    fn draw_with_brush<F, T: Interface>(&self, brush: &T, draw: F) -> WindowsResult<()>
    where
        F: Fn(&ID2D1HwndRenderTarget, &ID2D1Brush),
        T: Interface,
    {
        let brush_interface: ID2D1Brush = brush.cast()?;
        let target = self.target.as_ref().expect("No render target available");

        draw(target, &brush_interface);
        Ok(())
    }

    fn create_text_layout(&self, text: &str) -> WindowsResult<IDWriteTextLayout> {
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

    fn update_text_layout(&mut self, text: &str) -> Result<IDWriteTextLayout, OverlayError> {
        // Check cache first
        if let Some(layout) = self.text_layout_cache.get(text) {
            return Ok(layout.clone());
        }

        // Create new layout if not found
        let layout = self.create_text_layout(text)
            .map_err(|_| OverlayError::GetWriteTextFormatFailed)?;

        // Add to cache
        self.text_layout_cache.insert(text.to_string(), layout.clone());

        Ok(layout)
    }

    /// Call this in a render loop
    fn try_clear_text_layout_cache(&mut self) {
        self.cache_frame_count += 1;

        // Clear cache every 1000 frames or whatever number works best
        if self.cache_frame_count >= 72000 {
            self.text_layout_cache.clear();
            self.cache_frame_count = 0;
        }
    }

    fn create_solid_color_brush(&self, (r, g, b, a): (u8, u8, u8, u8)) -> WindowsResult<ID2D1SolidColorBrush> {
        let target = self.target.as_ref()
            .expect("No render target available");
        let brush_properties = create_brush_properties();
        let color = color_u8_to_f32((r, g, b, a));

        unsafe {
            // CreateSolidColorBrush returns Result<ID2D1SolidColorBrush>
            target.CreateSolidColorBrush(&color, Some(&brush_properties))
        }
    }

    /// Updates the color of the solid brush if it has changed
    fn update_solid_color_brush(
        &mut self,
        color: (u8, u8, u8, u8),
    ) -> Result<(), OverlayError> {
        let brush = self.solid_color_brush.as_ref()
            .ok_or(OverlayError::CreateSolidColorBrushFailed)?;

        unsafe {
            // Get current color
            let current_color = brush.GetColor();
            let new_color = color_u8_to_f32(color);

            if current_color != new_color {
                // Color changed, create new brush
                let target = self.target.as_ref()
                    .ok_or(OverlayError::NoRenderTarget)?;

                let new_brush = target
                    .CreateSolidColorBrush(
                        &new_color,
                        Some(&create_brush_properties())
                    )
                    .map_err(|_| OverlayError::CreateSolidColorBrushFailed)?;

                self.solid_color_brush = Some(new_brush);
            }
            // If color is the same, keep existing brush
        }

        Ok(())
    }

    fn create_linear_gradient_brush(
        &self,
        start_point: (f32, f32),
        end_point: (f32, f32),
        color1: (u8, u8, u8, u8),
        color2: (u8, u8, u8, u8),
    ) -> WindowsResult<ID2D1LinearGradientBrush> {
        let target = self.target.as_ref()
            .expect("No render target available");

        let gradient_stops = [
            D2D1_GRADIENT_STOP {
                position: 0.0,
                color: color_u8_to_f32(color1),
            },
            D2D1_GRADIENT_STOP {
                position: 1.0,
                color: color_u8_to_f32(color2),
            },
        ];

        unsafe {
            let gradient_stop_collection = target.CreateGradientStopCollection(
                &gradient_stops,
                D2D1_GAMMA_2_2,
                D2D1_EXTEND_MODE_CLAMP,
            )?;

            target.CreateLinearGradientBrush(
                &D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
                    startPoint: D2D_POINT_2F {
                        x: start_point.0,
                        y: start_point.1
                    },
                    endPoint: D2D_POINT_2F {
                        x: end_point.0,
                        y: end_point.1
                    },
                },
                Some(&create_brush_properties()),
                &gradient_stop_collection,
            )
        }
    }

    /// Compares gradient stops to see if they need to be updated, otherwise just modify start and end.
    fn update_linear_gradient_brush(
        &mut self,
        start: (f32, f32),
        end: (f32, f32),
        color1: (u8, u8, u8, u8),
        color2: (u8, u8, u8, u8),
    ) -> Result<(), OverlayError> {
        let brush = self.linear_gradient_brush.as_ref()
            .ok_or(OverlayError::CreateLinearGradientBrushFailed)?;

        unsafe {
            // Check current gradient stops
            let current_stops = brush.GetGradientStopCollection()
                .map_err(|_| OverlayError::CreateGradientStopCollectionFailed)?;

            // Create a buffer array to hold 2 gradient stops
            let mut stop_buffer = [D2D1_GRADIENT_STOP::default(); 2];

            // Fill our buffer with the current gradient stops data
            current_stops.GetGradientStops(&mut stop_buffer);

            // Extract the current colors and compare with new ones
            let current_start_color = stop_buffer[0].color;
            let current_end_color = stop_buffer[1].color;
            let new_start_color = color_u8_to_f32(color1);
            let new_end_color = color_u8_to_f32(color2);

            let needs_new_collection = current_start_color != new_start_color
                || current_end_color != new_end_color;

            if needs_new_collection {
                let target = self.target.as_ref()
                    .ok_or(OverlayError::NoRenderTarget)?;

                let gradient_stops = [
                    D2D1_GRADIENT_STOP {
                        position: 0.0,
                        color: color_u8_to_f32(color1),
                    },
                    D2D1_GRADIENT_STOP {
                        position: 1.0,
                        color: color_u8_to_f32(color2),
                    },
                ];

                let gradient_stop_collection = target
                    .CreateGradientStopCollection(
                        &gradient_stops,
                        D2D1_GAMMA_2_2,
                        D2D1_EXTEND_MODE_CLAMP,
                    )
                    .map_err(|_| OverlayError::CreateGradientStopCollectionFailed)?;

                let new_brush = target
                    .CreateLinearGradientBrush(
                        &D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
                            startPoint: D2D_POINT_2F { x: start.0, y: start.1 },
                            endPoint: D2D_POINT_2F { x: end.0, y: end.1 },
                        },
                        Some(&create_brush_properties()),
                        &gradient_stop_collection,
                    )
                    .map_err(|_| OverlayError::CreateLinearGradientBrushFailed)?;

                self.linear_gradient_brush = Some(new_brush);
            }
            else {
                // Just update the start and end points
                brush.SetStartPoint(D2D_POINT_2F { x: start.0, y: start.1 });
                brush.SetEndPoint(D2D_POINT_2F { x: end.0, y: end.1 });
            }
        }

        Ok(())
    }

    fn create_radial_gradient_brush(
        &self,
        center: (f32, f32),
        radius: (f32, f32),
        color1: (u8, u8, u8, u8),
        color2: (u8, u8, u8, u8),
    ) -> WindowsResult<ID2D1RadialGradientBrush> {
        let target = self.target.as_ref()
            .expect("No render target available");

        let gradient_stops = [
            D2D1_GRADIENT_STOP {
                position: 0.0,
                color: color_u8_to_f32(color1),
            },
            D2D1_GRADIENT_STOP {
                position: 1.0,
                color: color_u8_to_f32(color2),
            },
        ];

        unsafe {
            let gradient_stop_collection = target.CreateGradientStopCollection(
                &gradient_stops,
                D2D1_GAMMA_2_2,
                D2D1_EXTEND_MODE_CLAMP,
            )?;

            target.CreateRadialGradientBrush(
                &D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES {
                    center: D2D_POINT_2F {
                        x: center.0,
                        y: center.1
                    },
                    gradientOriginOffset: D2D_POINT_2F { x: 0.0, y: 0.0 },
                    radiusX: radius.0,
                    radiusY: radius.1,
                },
                Some(&create_brush_properties()),
                &gradient_stop_collection,
            )
        }
    }

    /// Compares gradient stops to see if they need to be updated, otherwise just modify start and end.
    fn update_radial_gradient_brush(
        &mut self,
        center: (f32, f32),
        radius: (f32, f32),
        color1: (u8, u8, u8, u8),
        color2: (u8, u8, u8, u8),
    ) -> Result<(), OverlayError> {
        let brush = self.radial_gradient_brush.as_ref()
            .ok_or(OverlayError::CreateRadialGradientBrushFailed)?;

        unsafe {
            let stops = brush.GetGradientStopCollection()
                .map_err(|_| OverlayError::CreateGradientStopCollectionFailed)?;

            // Create buffer and get current stops
            let mut stop_buffer = [D2D1_GRADIENT_STOP::default(); 2];
            stops.GetGradientStops(&mut stop_buffer);

            // Extract colors and compare
            let current_start_color = stop_buffer[0].color;
            let current_end_color = stop_buffer[1].color;
            let new_start_color = color_u8_to_f32(color1);
            let new_end_color = color_u8_to_f32(color2);

            if current_start_color != new_start_color || current_end_color != new_end_color {
                // Colors changed, create new brush
                let target = self.target.as_ref()
                    .ok_or(OverlayError::NoRenderTarget)?;

                let gradient_stops = [
                    D2D1_GRADIENT_STOP {
                        position: 0.0,
                        color: color_u8_to_f32(color1),
                    },
                    D2D1_GRADIENT_STOP {
                        position: 1.0,
                        color: color_u8_to_f32(color2),
                    },
                ];

                let gradient_stop_collection = target
                    .CreateGradientStopCollection(
                        &gradient_stops,
                        D2D1_GAMMA_2_2,
                        D2D1_EXTEND_MODE_CLAMP,
                    )
                    .map_err(|_| OverlayError::CreateGradientStopCollectionFailed)?;

                let new_brush = target
                    .CreateRadialGradientBrush(
                        &D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES {
                            center: D2D_POINT_2F { x: center.0, y: center.1 },
                            gradientOriginOffset: D2D_POINT_2F { x: 0.0, y: 0.0 },
                            radiusX: radius.0,
                            radiusY: radius.1,
                        },
                        Some(&create_brush_properties()),
                        &gradient_stop_collection,
                    )
                    .map_err(|_| OverlayError::CreateRadialGradientBrushFailed)?;

                self.radial_gradient_brush = Some(new_brush);
            }
            else {
                // Just update the center and radius
                brush.SetCenter(D2D_POINT_2F { x: center.0, y: center.1 });
                brush.SetRadiusX(radius.0);
                brush.SetRadiusY(radius.1);
            }
        }

        Ok(())
    }
}

// Auxiliary functions -----------------------------------------------------------------------------


pub fn find_target_window() -> Result<HWND, OverlayError> {
    // Encrypted strings for obscurity
    let nvidia_class_name: String = sc!("CEF-OSC-WIDGET\0", 120)
        .expect("Failed to decrypt string at compile time! (this should be impossible...)");
    let nvidia_window_name: String = sc!("NVIDIA GeForce Overlay\0", 121)
        .expect("Failed to decrypt string at compile time! (this should be impossible...)");
    let amd_class_name: String = sc!("AMDDVROVERLAYWINDOWCLASS\0", 120)
        .expect("Failed to decrypt string at compile time! (this should be impossible...)");
    let amd_window_name: String = sc!("amd dvr overlay\0", 121)
        .expect("Failed to decrypt string at compile time! (this should be impossible...)");

    unsafe {
        // Try to find first window
        let first_window = FindWindowA(
            PCSTR::from_raw(nvidia_class_name.as_ptr()),
            PCSTR::from_raw(nvidia_window_name.as_ptr()),
        );

        // Try to find second window
        let second_window = FindWindowA(
            PCSTR::from_raw(amd_class_name.as_ptr()),
            PCSTR::from_raw(amd_window_name.as_ptr()),
        );

        // Return first available window or error
        match (first_window, second_window) {
            (Ok(window), _) => Ok(window),
            (_, Ok(window)) => Ok(window),
            _ => Err(OverlayError::WindowNotFound),
        }
    }
}

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