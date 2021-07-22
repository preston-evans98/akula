pub mod header_merge_stream;
pub mod header_slices;

#[cfg(feature = "crossterm")]
pub mod ui_crossterm;
#[cfg(feature = "crossterm")]
pub use ui_crossterm::HeaderSlicesView;

#[cfg(not(feature = "crossterm"))]
pub mod ui_tracing;
#[cfg(not(feature = "crossterm"))]
pub use ui_tracing::HeaderSlicesView;
