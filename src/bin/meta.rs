use std::io::Write;

use anyhow::Result;

fn main() -> Result<()> {
    let output = luadot::lua::generate_definitions(std::env::args().skip(1))?;
    std::io::stdout().write_all(output.as_bytes())?;

    Ok(())
}
