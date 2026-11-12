//! The final image of a document (ADR 0004): one renderer for exports, Copy all,
//! thumbnails and everything the CLI does, identical on Windows and macOS.
//!
//! The editor canvas (Konva) draws the same geometry from `app/src/editor/geometry.ts`;
//! `tests/fixtures/arrow-geometry.json` keeps both copies in step.

pub mod callouts;
pub mod color;
pub mod cutout;
pub mod effects;
pub mod encode;
pub mod fx;
pub mod fx_marks;
pub mod fx_shapes;
pub mod fx_spot;
pub mod geometry;
pub mod highlights;
pub mod inpaint;
pub mod magnify;
pub mod paint;
pub mod pens;
pub mod pixels;
pub mod render;
pub mod selection;
pub mod shadows;
pub mod shapes;
pub mod simplify;
pub mod spotlights;
pub mod stamps;
pub mod steps;
pub mod text;

pub use render::{object_bounds, render, Base};
pub use tiny_skia::Transform;
