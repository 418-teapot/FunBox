//! Instruction set simulator.

pub mod execute;
pub mod isa;
pub mod regs;

// Test builds inject a `dead_code` allowance rejected by `forbid(unused)`.
#[cfg(not(test))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os();
    let _executable = args.next();

    if args.next().is_some() {
        return Err("iss takes no arguments; usage: iss".into());
    }

    isa::Isa::load()?;

    Ok(())
}
