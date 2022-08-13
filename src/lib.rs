//! Koi is a game engine that balances three primary qualities:
//! performance, ergonomics, and code simplicity.
//!
//! Changes are avoided that result in a small gain in
//! one quality while significantly regressing the others.
//!
//! 'Code simplicity' means that `Koi` avoids large and
//! complex dependencies that may introduce significant build times.

mod time;
use time::*;

mod app;
pub use app::*;

pub use koi_hierarchy::*;
pub use koi_transform::*;

pub use koi_ecs::*;
pub use koi_resources::*;

pub use koi_events::Event;

#[cfg(feature = "koi_renderer")]
pub use koi_renderer::*;
