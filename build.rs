type DynError = Box<dyn std::error::Error>;
type DynResult<T> = core::result::Result<T, DynError>;

fn main() -> DynResult<()> {
    println!("cargo:rerun-if-changed=src/powershell.pest");
    Ok(())
}
