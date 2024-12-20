use windows::Win32::Graphics::Direct2D::{
    D2D1_DRAW_TEXT_OPTIONS_NONE,
    D2D1_ELLIPSE,
    D2D1_ROUNDED_RECT,
    D2D1_CAP_STYLE,
    D2D1_STROKE_STYLE_PROPERTIES,
    D2D1_CAP_STYLE_FLAT,
    D2D1_LINE_JOIN,
    Common::{
        D2D_POINT_2F,
        D2D_RECT_F,
    },
};
use crate::helper::*;
use super::*;

impl Overlay {
    // TEXT -----------------------------------

    pub fn draw_text(
        &mut self,
        (x, y): (f32, f32),
        text: String,
        color: (u8, u8, u8, u8)
    ) -> Result <(), OverlayError> {
        self.update_solid_color_brush(color)?;

        // Get/update text layout from cache
        let text_layout = self.update_text_layout(text.as_str())?;

        // Get reference to the updated/cached brush
        let brush = self.solid_color_brush.as_ref()
            .ok_or(OverlayError::CreateSolidColorBrushFailed)?;

        self.draw_with_brush(
            brush, // Default to white if no color specified
            |target, brush| unsafe {
                target.DrawTextLayout(
                    D2D_POINT_2F { x, y },
                    &text_layout,
                    brush,
                    D2D1_DRAW_TEXT_OPTIONS_NONE,
                )
            }
        ).map_err(|_| OverlayError::DrawTextFailed(-1)) // Usually I'd use a match statement but this is already super nested.
    }

    pub fn draw_outlined_text(
        &mut self,
        (x, y): (f32, f32),
        text: String,
        color: (u8, u8, u8, u8)
    ) -> Result<(), OverlayError> {
        // Outline
        self.draw_text((x - 1.0, y), text.clone(), (0, 0, 0, 255))?;
        self.draw_text((x + 1.0, y), text.clone(), (0, 0, 0, 255))?;
        self.draw_text((x, y - 1.0), text.clone(), (0, 0, 0, 255))?;
        self.draw_text((x, y + 1.0), text.clone(), (0, 0, 0, 255))?;
        // Main text
        self.draw_text((x, y), text, color)?;

        Ok(())
    }

    // LINES ----------------------------------

    pub fn draw_line(
        &mut self,
        start: (f32, f32),
        end: (f32, f32),
        stroke_width: f32,
        color: (u8, u8, u8, u8),
    ) -> Result<(), OverlayError> {
        self.update_solid_color_brush(color)?;

        let brush = self.solid_color_brush.as_ref()
            .ok_or(OverlayError::CreateSolidColorBrushFailed)?;

        self.draw_with_brush(
            brush,
            |target, brush| unsafe {
                target.DrawLine(
                    D2D_POINT_2F { x: start.0, y: start.1 },
                    D2D_POINT_2F { x: end.0, y: end.1 },
                    brush,
                    stroke_width,
                    None,
                )
            }
        ).map_err(|_| OverlayError::DrawFailed)
    }

    pub fn draw_gradient_line(
        &mut self,
        start: (f32, f32),
        end: (f32, f32),
        stroke_width: f32,
        color1: (u8, u8, u8, u8),
        color2: (u8, u8, u8, u8),
    ) -> Result<(), OverlayError> {
        // Update cached brush if necessary
        self.update_linear_gradient_brush(
            start,
            end,
            color1,
            color2,
        )?;

        // Get reference to the updated/cached brush
        let brush = self.linear_gradient_brush.as_ref()
            .ok_or(OverlayError::CreateLinearGradientBrushFailed)?;

        // Use draw_with_brush for the actual rendering
        self.draw_with_brush(
            brush,
            |target, brush| unsafe {
                target.DrawLine(
                    D2D_POINT_2F { x: start.0, y: start.1 },
                    D2D_POINT_2F { x: end.0, y: end.1 },
                    brush,
                    stroke_width,
                    None,
                )
            }
        ).map_err(|_| OverlayError::DrawFailed)
    }

    pub fn draw_styled_line(
        &mut self,
        start: (f32, f32),
        end: (f32, f32),
        stroke_width: f32,
        color: (u8, u8, u8, u8),
        start_cap: D2D1_CAP_STYLE,
        end_cap: D2D1_CAP_STYLE,
    ) -> Result<(), OverlayError> {
        self.update_solid_color_brush(color)?;

        let brush = self.solid_color_brush.as_ref()
            .ok_or(OverlayError::CreateSolidColorBrushFailed)?;

        unsafe {
            let d2d_factory = self.d2d_factory.as_ref()
                .ok_or(OverlayError::NoD2DFactory)?;

            let stroke_style = d2d_factory.CreateStrokeStyle(
                &D2D1_STROKE_STYLE_PROPERTIES {
                    startCap: start_cap,
                    endCap: end_cap,
                    dashCap: D2D1_CAP_STYLE_FLAT,
                    lineJoin: D2D1_LINE_JOIN::default(),
                    miterLimit: 10.0,
                    dashStyle: Default::default(),
                    dashOffset: 0.0,
                },
                None,
            ).map_err(|_| OverlayError::CreateStrokeStyleFailed)?;

            self.draw_with_brush(
                brush,
                |target, brush|
                    target.DrawLine(
                        D2D_POINT_2F { x: start.0, y: start.1 },
                        D2D_POINT_2F { x: end.0, y: end.1 },
                        brush,
                        stroke_width,
                        Some(&stroke_style),
                    )
            ).map_err(|_| OverlayError::DrawFailed)
        }
    }

    // RECTANGLES ------------------------------

    pub fn draw_rect(
        &mut self,
        (x, y): (f32, f32),
        (width, height): (f32, f32),
        stroke_width: f32,
        color: (u8, u8, u8, u8)
    ) -> Result <(), OverlayError> {
        self.update_solid_color_brush(color)?;

        let rect = D2D_RECT_F {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        };

        let brush = self.solid_color_brush.as_ref()
            .ok_or(OverlayError::CreateSolidColorBrushFailed)?;

        self.draw_with_brush(
            brush, // Default to white if no color specified
            |target, brush| unsafe {
                target.DrawRectangle(&rect, brush, stroke_width, None)
            }
        ).map_err(|_| OverlayError::DrawFailed)
    }

    pub fn draw_filled_rect(
        &mut self,
        (x, y): (f32, f32),
        (width, height): (f32, f32),
        color: (u8, u8, u8, u8)
    ) -> Result <(), OverlayError> {
        self.update_solid_color_brush(color)?;

        let rect = D2D_RECT_F {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        };

        let brush = self.solid_color_brush.as_ref()
            .ok_or(OverlayError::CreateSolidColorBrushFailed)?;

        self.draw_with_brush(
            brush, // Default to white if no color specified
            |target, brush| unsafe {
                target.FillRectangle(&rect, brush)
            }
        ).map_err(|_| OverlayError::DrawFailed)
    }

    pub fn draw_gradient_rect(
        &mut self,
        (x, y): (f32, f32),
        (width, height): (f32, f32),
        color1: (u8, u8, u8, u8),
        color2: (u8, u8, u8, u8),
        is_vertical: bool,
    ) -> Result<(), OverlayError> {
        let (start, end) = if is_vertical {
            (
                (x, y),
                (x, y + height),
            )
        } else {
            (
                (x, y),
                (x + width, y),
            )
        };

        self.update_linear_gradient_brush(
            start,
            end,
            color1,
            color2,
        )?;

        let rect = D2D_RECT_F {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        };

        let brush = self.linear_gradient_brush.as_ref()
            .ok_or(OverlayError::CreateLinearGradientBrushFailed)?;

        self.draw_with_brush(
            brush,
            |target, brush| unsafe {
                target.FillRectangle(&rect, brush)
            }
        ).map_err(|_| OverlayError::DrawFailed)
    }

    // ROUNDED RECTANGLES ----------------------
    pub fn draw_rounded_rect(
        &mut self,
        (x, y): (f32, f32),
        (width, height): (f32, f32),
        radius: f32,
        stroke_width: f32,
        color: (u8, u8, u8, u8)
    ) -> Result<(), OverlayError> {
        self.update_solid_color_brush(color)?;

        let rect = D2D_RECT_F {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        };

        let rounded_rect = D2D1_ROUNDED_RECT {
            rect,
            radiusX: radius,
            radiusY: radius,
        };

        let brush = self.solid_color_brush.as_ref()
            .ok_or(OverlayError::CreateSolidColorBrushFailed)?;

        self.draw_with_brush(
            brush,
            |target, brush| unsafe {
                target.DrawRoundedRectangle(&rounded_rect, brush, stroke_width, None)
            }
        ).map_err(|_| OverlayError::DrawFailed)
    }

    pub fn draw_filled_rounded_rect(
        &mut self,
        (x, y): (f32, f32),
        (width, height): (f32, f32),
        radius: f32,
        color: (u8, u8, u8, u8)
    ) -> Result<(), OverlayError> {
        self.update_solid_color_brush(color)?;

        let rect = D2D_RECT_F {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        };

        let rounded_rect = D2D1_ROUNDED_RECT {
            rect,
            radiusX: radius,
            radiusY: radius,
        };

        let brush = self.solid_color_brush.as_ref()
            .ok_or(OverlayError::CreateSolidColorBrushFailed)?;

        self.draw_with_brush(
            brush,
            |target, brush| unsafe {
                target.FillRoundedRectangle(&rounded_rect, brush)
            }
        ).map_err(|_| OverlayError::DrawFailed)
    }

    pub fn draw_gradient_rounded_rect(
        &mut self,
        (x, y): (f32, f32),
        (width, height): (f32, f32),
        radius: f32,
        color1: (u8, u8, u8, u8),
        color2: (u8, u8, u8, u8),
        is_vertical: bool,
    ) -> Result<(), OverlayError> {
        let (start_point, end_point) = if is_vertical {
            (
                (x, y),
                (x, y + height),
            )
        } else {
            (
                (x, y),
                (x + width, y),
            )
        };

        self.update_linear_gradient_brush(
            start_point,
            end_point,
            color1,
            color2,
        )?;

        let rect = D2D_RECT_F {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        };

        let rounded_rect = D2D1_ROUNDED_RECT {
            rect,
            radiusX: radius,
            radiusY: radius,
        };

        let brush = self.linear_gradient_brush.as_ref()
            .ok_or(OverlayError::CreateLinearGradientBrushFailed)?;

        self.draw_with_brush(
            brush,
            |target, brush| unsafe {
                target.FillRoundedRectangle(&rounded_rect, brush)
            }
        ).map_err(|_| OverlayError::DrawFailed)
    }

    // CIRCLES ---------------------------------
    pub fn draw_circle(
        &mut self,
        center: (f32, f32),
        radius: f32,
        stroke_width: f32,
        color: (u8, u8, u8, u8)
    ) -> Result<(), OverlayError> {
        self.update_solid_color_brush(color)?;

        let ellipse = D2D1_ELLIPSE {
            point: D2D_POINT_2F {
                x: center.0,
                y: center.1,
            },
            radiusX: radius,
            radiusY: radius,
        };

        let brush = self.solid_color_brush.as_ref()
            .ok_or(OverlayError::CreateSolidColorBrushFailed)?;

        self.draw_with_brush(
            brush,
            |target, brush| unsafe {
                target.DrawEllipse(&ellipse, brush, stroke_width, None)
            }
        ).map_err(|_| OverlayError::DrawFailed)
    }

    pub fn draw_filled_circle(
        &mut self,
        center: (f32, f32),
        radius: f32,
        color: (u8, u8, u8, u8)
    ) -> Result<(), OverlayError> {
        self.update_solid_color_brush(color)?;

        let ellipse = D2D1_ELLIPSE {
            point: D2D_POINT_2F {
                x: center.0,
                y: center.1,
            },
            radiusX: radius,
            radiusY: radius,
        };

        let brush = self.solid_color_brush.as_ref()
            .ok_or(OverlayError::CreateSolidColorBrushFailed)?;

        self.draw_with_brush(
            brush,
            |target, brush| unsafe {
                target.FillEllipse(&ellipse, brush)
            }
        ).map_err(|_| OverlayError::DrawFailed)
    }

    pub fn draw_gradient_circle(
        &mut self,
        center: (f32, f32),
        radius: f32,
        color1: (u8, u8, u8, u8),
        color2: (u8, u8, u8, u8),
        is_radial: bool,
    ) -> Result<(), OverlayError> {
        if is_radial {
            self.update_radial_gradient_brush(
                center,
                (radius, radius),
                color1,
                color2,
            )?;

            let brush = self.radial_gradient_brush.as_ref()
                .ok_or(OverlayError::CreateRadialGradientBrushFailed)?;

            let ellipse = D2D1_ELLIPSE {
                point: D2D_POINT_2F {
                    x: center.0,
                    y: center.1,
                },
                radiusX: radius,
                radiusY: radius,
            };

            self.draw_with_brush(
                brush,
                |target, brush| unsafe {
                    target.FillEllipse(&ellipse, brush)
                }
            )
        } else {
            self.update_linear_gradient_brush(
                (center.0 - radius, center.1),
                (center.0 + radius, center.1),
                color1,
                color2,
            )?;

            let brush = self.linear_gradient_brush.as_ref()
                .ok_or(OverlayError::CreateLinearGradientBrushFailed)?;

            let ellipse = D2D1_ELLIPSE {
                point: D2D_POINT_2F {
                    x: center.0,
                    y: center.1,
                },
                radiusX: radius,
                radiusY: radius,
            };

            self.draw_with_brush(
                brush,
                |target, brush| unsafe {
                    target.FillEllipse(&ellipse, brush)
                }
            )
        }.map_err(|_| OverlayError::DrawFailed)
    }

    // ELLIPSES -------------------------------
    pub fn draw_ellipse(
        &mut self,
        center: (f32, f32),
        (radius_x, radius_y): (f32, f32),
        stroke_width: f32,
        color: (u8, u8, u8, u8)
    ) -> Result<(), OverlayError> {
        self.update_solid_color_brush(color)?;

        let ellipse = D2D1_ELLIPSE {
            point: D2D_POINT_2F {
                x: center.0,
                y: center.1,
            },
            radiusX: radius_x,
            radiusY: radius_y,
        };

        let brush = self.solid_color_brush.as_ref()
            .ok_or(OverlayError::CreateSolidColorBrushFailed)?;

        self.draw_with_brush(
            brush,
            |target, brush| unsafe {
                target.DrawEllipse(&ellipse, brush, stroke_width, None)
            }
        ).map_err(|_| OverlayError::DrawFailed)
    }

    pub fn draw_filled_ellipse(
        &mut self,
        center: (f32, f32),
        (radius_x, radius_y): (f32, f32),
        color: (u8, u8, u8, u8)
    ) -> Result<(), OverlayError> {
        self.update_solid_color_brush(color)?;

        let ellipse = D2D1_ELLIPSE {
            point: D2D_POINT_2F {
                x: center.0,
                y: center.1,
            },
            radiusX: radius_x,
            radiusY: radius_y,
        };

        let brush = self.solid_color_brush.as_ref()
            .ok_or(OverlayError::CreateSolidColorBrushFailed)?;

        self.draw_with_brush(
            brush,
            |target, brush| unsafe {
                target.FillEllipse(&ellipse, brush)
            }
        ).map_err(|_| OverlayError::DrawFailed)
    }

    pub fn draw_gradient_ellipse(
        &mut self,
        center: (f32, f32),
        (radius_x, radius_y): (f32, f32),
        color1: (u8, u8, u8, u8),
        color2: (u8, u8, u8, u8),
        is_radial: bool,
    ) -> Result<(), OverlayError> {
        if is_radial {
            self.update_radial_gradient_brush(
                center,
                (radius_x, radius_y),
                color1,
                color2,
            )?;

            let brush = self.radial_gradient_brush.as_ref()
                .ok_or(OverlayError::CreateRadialGradientBrushFailed)?;

            let ellipse = D2D1_ELLIPSE {
                point: D2D_POINT_2F {
                    x: center.0,
                    y: center.1,
                },
                radiusX: radius_x,
                radiusY: radius_y,
            };

            self.draw_with_brush(
                brush,
                |target, brush| unsafe {
                    target.FillEllipse(&ellipse, brush)
                }
            )
        } else {
            self.update_linear_gradient_brush(
                (center.0 - radius_x, center.1),
                (center.0 + radius_x, center.1),
                color1,
                color2,
            )?;

            let brush = self.linear_gradient_brush.as_ref()
                .ok_or(OverlayError::CreateLinearGradientBrushFailed)?;

            let ellipse = D2D1_ELLIPSE {
                point: D2D_POINT_2F {
                    x: center.0,
                    y: center.1,
                },
                radiusX: radius_x,
                radiusY: radius_y,
            };

            self.draw_with_brush(
                brush,
                |target, brush| unsafe {
                    target.FillEllipse(&ellipse, brush)
                }
            )
        }.map_err(|_| OverlayError::DrawFailed)
    }
}