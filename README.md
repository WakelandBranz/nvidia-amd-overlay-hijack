# Overlay Hijack

Credits to [WilgnerFSDev](https://github.com/WilgnerFSDev) for creating the foundations of this project.

A Rust library for hijacking the NVIDIA GeForce Experience and AMD Radeon overlays for custom rendering.

Showcase: https://youtu.be/MKifNl3XCHQ

***Most code based off*** https://github.com/WilgnerFSDev/nvidia-overlay-hijack-rs/

## Basic Example
```rust
use nvidia_overlay_hijack::{Overlay, OverlayError};

fn main() -> Result<(), OverlayError> {
    // Create a new overlay with Segoe UI font at 18pt
    let mut overlay = Overlay::new("Segoe UI", 18.0);
    
    // Initialize the overlay window
    overlay.init()?;
    overlay.startup_d2d()?;
    
    // Begin drawing
    overlay.begin_scene();
    overlay.clear_scene();
    
    // Draw some text (red color)
    overlay.draw_text(
        (10.0, 30.0),
        "Hello World!",
        (255, 0, 0, 255)
    )?;
    
    // End drawing
    overlay.end_scene();
    
    Ok(())
}
```

## Available Drawing Functions

### Basic Shapes
```rust
// Text
overlay.draw_text((x, y), "Text to draw", color)?;

// Basic line
overlay.draw_line(start_point, end_point, stroke_width, color)?;

// Gradient line
overlay.draw_gradient_line(start_point, end_point, stroke_width, color1, color2)?;

// Styled line (dashed, dotted, etc.)
overlay.draw_styled_line(
    start_point,
    end_point,
    stroke_width,
    color,
    dash_style,    // D2D1_DASH_STYLE
    start_cap,     // D2D1_CAP_STYLE
    end_cap        // D2D1_CAP_STYLE
)?;

// Rectangles
overlay.draw_rect((x, y), (width, height), stroke_width, color)?;
overlay.draw_filled_rect((x, y), (width, height), color)?;
overlay.draw_gradient_rect((x, y), (width, height), color1, color2, is_vertical)?;

// Rounded Rectangles
overlay.draw_rounded_rect((x, y), (width, height), radius, stroke_width, color)?;
overlay.draw_filled_rounded_rect((x, y), (width, height), radius, color)?;
overlay.draw_gradient_rounded_rect((x, y), (width, height), radius, color1, color2, is_vertical)?;

// Circles
overlay.draw_circle(center, radius, stroke_width, color)?;
overlay.draw_filled_circle(center, radius, color)?;
overlay.draw_gradient_circle(center, radius, color1, color2, is_radial)?;

// Ellipses
overlay.draw_ellipse(center, (radius_x, radius_y), stroke_width, color)?;
overlay.draw_filled_ellipse(center, (radius_x, radius_y), color)?;
overlay.draw_gradient_ellipse(center, (radius_x, radius_y), color1, color2, is_radial)?;
```

### Colors and Gradients
- Colors are specified as RGBA tuples: `(r, g, b, a)` where each component is 0-255
- Gradients support both linear and radial modes:
    - For rectangles: `is_vertical` determines gradient direction
    - For circles/ellipses: `is_radial` switches between radial and linear gradients

### Line Styles
Available dash styles:
- D2D1_DASH_STYLE_SOLID
- D2D1_DASH_STYLE_DASH
- D2D1_DASH_STYLE_DOT
- D2D1_DASH_STYLE_DASH_DOT
- D2D1_DASH_STYLE_DASH_DOT_DOT

Available cap styles:
- D2D1_CAP_STYLE_FLAT
- D2D1_CAP_STYLE_ROUND
- D2D1_CAP_STYLE_SQUARE

### Example with Multiple Shapes
```rust
fn main() -> Result<(), OverlayError> {
    let mut overlay = Overlay::new("Segoe UI", 18.0);
    overlay.init()?;
    overlay.startup_d2d()?;

    loop {
        overlay.begin_scene();
        overlay.clear_scene();

        // Draw text
        overlay.draw_text(
            (10.0, 30.0),
            "Overlay Example",
            (255, 255, 255, 255)
        )?;

        // Draw a filled rectangle with gradient
        overlay.draw_gradient_rect(
            (10.0, 50.0),
            (200.0, 100.0),
            (255, 0, 0, 255),   // Start color (red)
            (0, 0, 255, 255),   // End color (blue)
            true                // Vertical gradient
        )?;

        // Draw a circle with outline
        overlay.draw_circle(
            (300.0, 100.0),     // Center point
            50.0,               // Radius
            2.0,                // Stroke width
            (0, 255, 0, 255)    // Color (green)
        )?;

        overlay.end_scene();
    }

    Ok(())
}
```

## Error Handling
The library uses a custom `OverlayError` type that covers various failure cases:
- Window creation/manipulation errors
- Direct2D initialization errors
- Drawing errors
- Gradient creation errors

## Notes
- Requires NVIDIA GeForce Experience or AMD Radeon Overlay to be installed and running. This has not yet been tested on AMD's Radeon Overlay.
- Performance depends on system capabilities
- Alpha blending is supported for all shapes and gradients
