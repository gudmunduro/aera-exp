use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(&["protobuf/tcp_data_message.proto"], &["protobuf/"])?;
    Ok(())
}