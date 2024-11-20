# NVIDIA Overlay Hijacking

Based off https://github.com/WilgnerFSDev/nvidia-overlay-hijack-rs/

A safe Rust implementation for manipulating the NVIDIA overlay window using windows-rs. This project provides a clean API for drawing custom shapes, text, and other elements on top of the NVIDIA GeForce overlay.

This crate uses windows-rs for safe Windows API bindings and proper resource management.

## Usage
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
       Some((255, 0, 0, 255))
   )?;
   
   // Draw a rectangle
   overlay.draw_rect(
       (10.0, 80.0),
       (100.0, 100.0),
       2.0,
       None  // Default white color
   )?;
   
   overlay.end_scene();
   
   Ok(())
}
