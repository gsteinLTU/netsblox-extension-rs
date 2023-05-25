use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    netsblox_extension_util::build()
}