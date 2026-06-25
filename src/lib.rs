#![cfg_attr(test, allow(clippy::unwrap_used))]

pub mod adapters;
pub mod application;
pub mod domain;
pub mod interfaces;

pub use domain::errors::{CellError, CellResult};
