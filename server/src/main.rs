use anyhow::Result;
use esp_idf_sys::link_patches;

fn main() -> Result<()> {
    link_patches();

    Ok(())
}
