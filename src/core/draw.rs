use windows::Win32::Graphics::Direct2D::Common::{D2D_POINT_2F, D2D_RECT_F};
use windows::Win32::Graphics::Direct2D::{D2D1_DRAW_TEXT_OPTIONS_NONE, D2D1_ELLIPSE};
use crate::helper::*;
use super::*;

impl Overlay {
    pub fn draw_text(
        &mut self,
        (x, y): (f32, f32),
        text: String,
        color: Option<(u8, u8, u8, u8)>
    ) -> Result <(), OverlayError> {
        let text_layout = self.create_text_layout(&text).expect("Failed to get text_layout");

        self.draw_element(
            color.unwrap_or((255, 255, 255, 255)), // Default to white if no color specified
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

    pub fn draw_rect(
        &mut self,
        (x, y): (f32, f32),
        (width, height): (f32, f32),
        stroke_width: f32,
        color: (u8, u8, u8, u8)
    ) -> Result <(), OverlayError> {
        let rect = D2D_RECT_F {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        };

        // This might be useful for the future if I want to make custom box rendering functions.

        // Create stroke style properties
        //let stroke_properties = D2D1_STROKE_STYLE_PROPERTIES {
        //    startCap: D2D1_CAP_STYLE::D2D1_CAP_STYLE_SQUARE,   // or FLAT, SQUARE
        //    endCap: D2D1_CAP_STYLE::D2D1_CAP_STYLE_ROUND,
        //    dashCap: D2D1_CAP_STYLE::D2D1_CAP_STYLE_ROUND,
        //    lineJoin: D2D1_LINE_JOIN::D2D1_LINE_JOIN_ROUND,   // or MITER, BEVEL
        //    miterLimit: 10.0,
        //    dashStyle: D2D1_DASH_STYLE::D2D1_DASH_STYLE_SOLID, // or DASH, DOT, DASH_DOT, etc.
        //    dashOffset: 0.0,
        //};

        self.draw_element(
            color, // Default to white if no color specified
            |target, brush| unsafe {
                target.DrawRectangle(&rect, brush, stroke_width, None)
            }
        ).map_err(|_| OverlayError::DrawTextFailed(-1)) // Usually I'd use a match statement but this is already super nested.
    }

    pub fn draw_circle(
        &mut self,
        center: (f32, f32),  // Center point instead of top-left
        radius: f32,         // Single radius value for circle
        stroke_width: f32,
        color: (u8, u8, u8, u8)
    ) -> Result<(), OverlayError> {
        let ellipse = D2D1_ELLIPSE {
            point: D2D_POINT_2F {
                x: center.0,
                y: center.1,
            },
            radiusX: radius,
            radiusY: radius,  // Same radius for both axes makes it a circle
        };

        self.draw_element(
            color,
            |target, brush| unsafe {
                target.DrawEllipse(&ellipse, brush, stroke_width, None)
            }
        ).map_err(|_| OverlayError::DrawTextFailed(-1))
    }
}