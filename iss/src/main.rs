//! Instruction set simulator.

pub mod regs;

// Test builds inject a `dead_code` allowance rejected by `forbid(unused)`.
#[cfg(not(test))]
fn main() -> std::process::ExitCode {
    let mut args = std::env::args_os();
    let _executable = args.next();

    if args.next().is_some() {
        eprintln!("usage: iss");
        eprintln!("iss currently accepts no arguments");

        return std::process::ExitCode::FAILURE;
    }

    std::process::ExitCode::SUCCESS
}
