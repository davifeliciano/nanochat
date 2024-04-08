use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(&["src/chat.proto"], &["src/"])?;
    Ok(())
}
