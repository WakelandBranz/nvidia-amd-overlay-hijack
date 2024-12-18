use windows::Win32::Graphics::Direct2D::{
    D2D1_DRAW_TEXT_OPTIONS_NONE,
    D2D1_ELLIPSE,
    D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES,
    D2D1_GAMMA_2_2, D2D1_EXTEND_MODE_CLAMP,
    D2D1_ROUNDED_RECT,
    D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES,
    D2D1_CAP_STYLE,
    D2D1_STROKE_STYLE_PROPERTIES,
    D2D1_CAP_STYLE_FLAT,
    D2D1_LINE_JOIN,
    Common::{
        D2D1_GRADIENT_STOP,
        D2D_POINT_2F,
        D2D_RECT_F,
        D2D1_COLOR_F,
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
        let text_layout = self.create_text_layout(&text).expect("Failed to get text_layout");

        self.draw_element(
            color, // Default to white if no color specified
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

    // LINES ----------------------------------

    pub fn draw_line(
        &mut self,
        start: (f32, f32),
        end: (f32, f32),
        stroke_width: f32,
        color: (u8, u8, u8, u8),
    ) -> Result<(), OverlayError> {
        self.draw_element(
            color,
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
        // Convert colors to D2D1_COLOR_F format
        let start_color = D2D1_COLOR_F {
            r: color1.0 as f32 / 255.0,
            g: color1.1 as f32 / 255.0,
            b: color1.2 as f32 / 255.0,
            a: color1.3 as f32 / 255.0,
        };
        let end_color = D2D1_COLOR_F {
            r: color2.0 as f32 / 255.0,
            g: color2.1 as f32 / 255.0,
            b: color2.2 as f32 / 255.0,
            a: color2.3 as f32 / 255.0,
        };

        let gradient_stops = [
            D2D1_GRADIENT_STOP {
                position: 0.0,
                color: start_color,
            },
            D2D1_GRADIENT_STOP {
                position: 1.0,
                color: end_color,
            },
        ];

        unsafe {
            let render_target = self.target.as_ref()
                .ok_or(OverlayError::NoRenderTarget)?;

            let gradient_stop_collection = render_target
                .CreateGradientStopCollection(
                    &gradient_stops,
                    D2D1_GAMMA_2_2,
                    D2D1_EXTEND_MODE_CLAMP,
                )
                .map_err(|_| OverlayError::CreateGradientStopCollectionFailed)?;

            let gradient_brush = render_target
                .CreateLinearGradientBrush(
                    &D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
                        startPoint: D2D_POINT_2F { x: start.0, y: start.1 },
                        endPoint: D2D_POINT_2F { x: end.0, y: end.1 },
                    },
                    None,
                    &gradient_stop_collection,
                )
                .map_err(|_| OverlayError::CreateLinearGradientBrushFailed)?;

            render_target.DrawLine(
                D2D_POINT_2F { x: start.0, y: start.1 },
                D2D_POINT_2F { x: end.0, y: end.1 },
                &gradient_brush,
                stroke_width,
                None,
            );

            Ok(())
        }
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

            self.draw_element(
                color,
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
        let rect = D2D_RECT_F {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        };

        self.draw_element(
            color, // Default to white if no color specified
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
        let rect = D2D_RECT_F {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        };

        self.draw_element(
            color, // Default to white if no color specified
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
        let rect = D2D_RECT_F {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        };

        // Convert colors to D2D1_COLOR_F format (0.0 to 1.0 range)
        let start_color = D2D1_COLOR_F {
            r: color1.0 as f32 / 255.0,
            g: color1.1 as f32 / 255.0,
            b: color1.2 as f32 / 255.0,
            a: color1.3 as f32 / 255.0,
        };
        let end_color = D2D1_COLOR_F {
            r: color2.0 as f32 / 255.0,
            g: color2.1 as f32 / 255.0,
            b: color2.2 as f32 / 255.0,
            a: color2.3 as f32 / 255.0,
        };

        // Define gradient stops
        let gradient_stops = [
            D2D1_GRADIENT_STOP {
                position: 0.0,
                color: start_color,
            },
            D2D1_GRADIENT_STOP {
                position: 1.0,
                color: end_color,
            },
        ];

        // Define start and end points for the gradient
        let (start_point, end_point) = if is_vertical {
            (
                D2D_POINT_2F { x, y },
                D2D_POINT_2F { x, y: y + height },
            )
        }
        else {
            (
                D2D_POINT_2F { x, y },
                D2D_POINT_2F { x: x + width, y },
            )
        };

        unsafe {
            let render_target = self.target.as_ref()
                .ok_or(OverlayError::NoRenderTarget)?;

            // Create gradient stop collection
            let gradient_stop_collection = render_target
                .CreateGradientStopCollection(
                    &gradient_stops,
                    D2D1_GAMMA_2_2,
                    D2D1_EXTEND_MODE_CLAMP,
                )
                .map_err(|_| OverlayError::CreateGradientStopCollectionFailed)?;

            // Create linear gradient brush
            let gradient_brush = render_target
                .CreateLinearGradientBrush(
                    &D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
                        startPoint: start_point,
                        endPoint: end_point,
                    },
                    None,
                    &gradient_stop_collection,
                )
                .map_err(|_| OverlayError::CreateLinearGradientBrushFailed)?;

            // Fill rectangle with gradient
            render_target.FillRectangle(&rect, &gradient_brush);

            Ok(())
        }
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

        self.draw_element(
            color,
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

        self.draw_element(
            color,
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

        // Convert colors to D2D1_COLOR_F format (0.0 to 1.0 range)
        let start_color = D2D1_COLOR_F {
            r: color1.0 as f32 / 255.0,
            g: color1.1 as f32 / 255.0,
            b: color1.2 as f32 / 255.0,
            a: color1.3 as f32 / 255.0,
        };
        let end_color = D2D1_COLOR_F {
            r: color2.0 as f32 / 255.0,
            g: color2.1 as f32 / 255.0,
            b: color2.2 as f32 / 255.0,
            a: color2.3 as f32 / 255.0,
        };

        let gradient_stops = [
            D2D1_GRADIENT_STOP {
                position: 0.0,
                color: start_color,
            },
            D2D1_GRADIENT_STOP {
                position: 1.0,
                color: end_color,
            },
        ];

        let (start_point, end_point) = if is_vertical {
            (
                D2D_POINT_2F { x, y },
                D2D_POINT_2F { x, y: y + height },
            )
        }
        else {
            (
                D2D_POINT_2F { x, y },
                D2D_POINT_2F { x: x + width, y },
            )
        };

        unsafe {
            let render_target = self.target.as_ref()
                .ok_or(OverlayError::NoRenderTarget)?;

            let gradient_stop_collection = render_target
                .CreateGradientStopCollection(
                    &gradient_stops,
                    D2D1_GAMMA_2_2,
                    D2D1_EXTEND_MODE_CLAMP,
                )
                .map_err(|_| OverlayError::CreateGradientStopCollectionFailed)?;

            let gradient_brush = render_target
                .CreateLinearGradientBrush(
                    &D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
                        startPoint: start_point,
                        endPoint: end_point,
                    },
                    None,
                    &gradient_stop_collection,
                )
                .map_err(|_| OverlayError::CreateLinearGradientBrushFailed)?;

            render_target.FillRoundedRectangle(&rounded_rect, &gradient_brush);

            Ok(())
        }
    }

    // CIRCLES ---------------------------------

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

    pub fn draw_filled_circle(
        &mut self,
        center: (f32, f32),  // Center point instead of top-left
        radius: f32,         // Single radius value for circle
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
                target.FillEllipse(&ellipse, brush)
            }
        ).map_err(|_| OverlayError::DrawTextFailed(-1))
    }

    pub fn draw_gradient_circle(
        &mut self,
        center: (f32, f32),
        radius: f32,
        color1: (u8, u8, u8, u8),
        color2: (u8, u8, u8, u8),
        is_radial: bool, // true for radial gradient, false for linear
    ) -> Result<(), OverlayError> {
        let ellipse = D2D1_ELLIPSE {
            point: D2D_POINT_2F {
                x: center.0,
                y: center.1,
            },
            radiusX: radius,
            radiusY: radius,
        };

        // Convert colors to D2D1_COLOR_F format
        let start_color = D2D1_COLOR_F {
            r: color1.0 as f32 / 255.0,
            g: color1.1 as f32 / 255.0,
            b: color1.2 as f32 / 255.0,
            a: color1.3 as f32 / 255.0,
        };
        let end_color = D2D1_COLOR_F {
            r: color2.0 as f32 / 255.0,
            g: color2.1 as f32 / 255.0,
            b: color2.2 as f32 / 255.0,
            a: color2.3 as f32 / 255.0,
        };

        let gradient_stops = [
            D2D1_GRADIENT_STOP {
                position: 0.0,
                color: start_color,
            },
            D2D1_GRADIENT_STOP {
                position: 1.0,
                color: end_color,
            },
        ];

        unsafe {
            let render_target = self.target.as_ref()
                .ok_or(OverlayError::NoRenderTarget)?;

            let gradient_stop_collection = render_target
                .CreateGradientStopCollection(
                    &gradient_stops,
                    D2D1_GAMMA_2_2,
                    D2D1_EXTEND_MODE_CLAMP,
                )
                .map_err(|_| OverlayError::CreateGradientStopCollectionFailed)?;

            if is_radial {
                // Create radial gradient brush
                let radial_brush = render_target
                    .CreateRadialGradientBrush(
                        &D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES {
                            center: D2D_POINT_2F {
                                x: center.0,
                                y: center.1,
                            },
                            gradientOriginOffset: D2D_POINT_2F {
                                x: 0.0,
                                y: 0.0,
                            },
                            radiusX: radius,
                            radiusY: radius,
                        },
                        None,
                        &gradient_stop_collection,
                    )
                    .map_err(|_| OverlayError::CreateRadialGradientBrushFailed)?;

                render_target.FillEllipse(&ellipse, &radial_brush);
            }
            else {
                // Create linear gradient brush
                let linear_brush = render_target
                    .CreateLinearGradientBrush(
                        &D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
                            startPoint: D2D_POINT_2F {
                                x: center.0 - radius,
                                y: center.1,
                            },
                            endPoint: D2D_POINT_2F {
                                x: center.0 + radius,
                                y: center.1,
                            },
                        },
                        None,
                        &gradient_stop_collection,
                    )
                    .map_err(|_| OverlayError::CreateLinearGradientBrushFailed)?;

                render_target.FillEllipse(&ellipse, &linear_brush);
            }

            Ok(())
        }
    }

    // ELLIPSES --------------------------------

    pub fn draw_ellipse(
        &mut self,
        center: (f32, f32),
        (radius_x, radius_y): (f32, f32),
        stroke_width: f32,
        color: (u8, u8, u8, u8)
    ) -> Result<(), OverlayError> {
        let ellipse = D2D1_ELLIPSE {
            point: D2D_POINT_2F {
                x: center.0,
                y: center.1,
            },
            radiusX: radius_x,
            radiusY: radius_y,
        };

        self.draw_element(
            color,
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
        let ellipse = D2D1_ELLIPSE {
            point: D2D_POINT_2F {
                x: center.0,
                y: center.1,
            },
            radiusX: radius_x,
            radiusY: radius_y,
        };

        self.draw_element(
            color,
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
        let ellipse = D2D1_ELLIPSE {
            point: D2D_POINT_2F {
                x: center.0,
                y: center.1,
            },
            radiusX: radius_x,
            radiusY: radius_y,
        };

        // Convert colors to D2D1_COLOR_F format
        let start_color = D2D1_COLOR_F {
            r: color1.0 as f32 / 255.0,
            g: color1.1 as f32 / 255.0,
            b: color1.2 as f32 / 255.0,
            a: color1.3 as f32 / 255.0,
        };
        let end_color = D2D1_COLOR_F {
            r: color2.0 as f32 / 255.0,
            g: color2.1 as f32 / 255.0,
            b: color2.2 as f32 / 255.0,
            a: color2.3 as f32 / 255.0,
        };

        let gradient_stops = [
            D2D1_GRADIENT_STOP {
                position: 0.0,
                color: start_color,
            },
            D2D1_GRADIENT_STOP {
                position: 1.0,
                color: end_color,
            },
        ];

        unsafe {
            let render_target = self.target.as_ref()
                .ok_or(OverlayError::NoRenderTarget)?;

            let gradient_stop_collection = render_target
                .CreateGradientStopCollection(
                    &gradient_stops,
                    D2D1_GAMMA_2_2,
                    D2D1_EXTEND_MODE_CLAMP,
                )
                .map_err(|_| OverlayError::CreateGradientStopCollectionFailed)?;

            if is_radial {
                // Create radial gradient brush
                let radial_brush = render_target
                    .CreateRadialGradientBrush(
                        &D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES {
                            center: D2D_POINT_2F {
                                x: center.0,
                                y: center.1,
                            },
                            gradientOriginOffset: D2D_POINT_2F {
                                x: 0.0,
                                y: 0.0,
                            },
                            radiusX: radius_x,
                            radiusY: radius_y,
                        },
                        None,
                        &gradient_stop_collection,
                    )
                    .map_err(|_| OverlayError::CreateRadialGradientBrushFailed)?;

                render_target.FillEllipse(&ellipse, &radial_brush);
            }
            else {
                // Create linear gradient brush
                let linear_brush = render_target
                    .CreateLinearGradientBrush(
                        &D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
                            startPoint: D2D_POINT_2F {
                                x: center.0 - radius_x,
                                y: center.1,
                            },
                            endPoint: D2D_POINT_2F {
                                x: center.0 + radius_x,
                                y: center.1,
                            },
                        },
                        None,
                        &gradient_stop_collection,
                    )
                    .map_err(|_| OverlayError::CreateLinearGradientBrushFailed)?;

                render_target.FillEllipse(&ellipse, &linear_brush);
            }

            Ok(())
        }
    }
}