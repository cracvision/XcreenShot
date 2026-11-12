//! The final image of a document (ADR 0004): one renderer for exports, Copy all,
//! thumbnails and everything the CLI does, identical on Windows and macOS.
//!
//! The editor canvas (Konva) draws the same geometry from `app/src/editor/geometry.ts`;
//! `tests/fixtures/arrow-geometry.json` keeps both copies in step.

pub mod callouts;
pub mod color;
pub mod effects;
pub mod encode;
pub mod geometry;
pub mod highlights;
pub mod magnify;
pub mod pens;
pub mod render;
pub mod shapes;
pub mod steps;
pub mod text;

pub use render::{object_bounds, render, Base};
