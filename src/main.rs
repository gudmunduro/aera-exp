pub mod types;
mod runtime;
pub mod interfaces;
pub mod utils;

pub mod protobuf {
    include!(concat!(env!("OUT_DIR"), "/tcp_io_device.rs"));
}

fn main() {
    setup_logging();

    runtime::run_demo();
}

fn setup_logging() {
    simple_log::quick!();
}