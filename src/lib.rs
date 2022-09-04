//! Koi is a game engine that balances three primary qualities:
//! performance, ergonomics, and code simplicity.
//!
//! Changes are avoided that result in a small gain in
//! one quality while significantly regressing the others.
//!
//! 'Code simplicity' means that `koi` avoids large and
//! complex dependencies that may introduce significant build times.

mod app;
pub use app::*;

pub use kmath::*;
pub use koi_ecs::*;
pub use koi_resources::*;
pub use koi_time::*;
pub use koi_transform::*;

pub use kapp_platform_common::Event as KappEvent;
pub use kapp_platform_common::{Cursor, Key, PointerButton, PointerSource};
pub use koi_animation::*;
pub use koi_assets::*;
pub use koi_events::Event;
pub use koi_prefabs::*;

#[cfg(feature = "koi_renderer")]
pub use koi_renderer::*;

#[cfg(feature = "koi_input")]
pub use koi_input::*;

#[cfg(feature = "koi_random")]
pub use koi_random::*;

#[cfg(feature = "koi_audio")]
pub use koi_audio::*;

pub mod scripts {
    pub use koi_camera_controls::*;
}

pub use koi_ecs;
