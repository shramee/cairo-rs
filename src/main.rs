#![deny(warnings)]
mod serde;
mod types;
mod utils;
mod vm;

use tracing::info;

fn main() {
    utils::setup_tracing();
    info!("hello world");
}
