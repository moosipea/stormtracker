#![warn(clippy::all, rust_2018_idioms)]

pub mod app;
pub mod server;
pub use app::StormtrackerApp;
pub use server::*;