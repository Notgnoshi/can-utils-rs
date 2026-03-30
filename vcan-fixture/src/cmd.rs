use std::process::Output;

pub use assert_cmd::Command;

pub trait CommandExt {
    /// Same as [Command::output] except with hooks to print stdout/stderr in failed tests
    fn captured_output(&mut self) -> std::io::Result<Output>;
}

impl CommandExt for Command {
    fn captured_output(&mut self) -> std::io::Result<Output> {
        let output = self.output()?;

        // libtest injects magic in print! macros to capture output in tests
        print!("{}", String::from_utf8_lossy(&output.stdout));
        eprint!("{}", String::from_utf8_lossy(&output.stderr));

        Ok(output)
    }
}

/// Get a command to run the given tool binary.
///
/// Uses `CARGO_BIN_EXE_<name>` which cargo sets at compile time for integration tests in the same
/// crate as the binary.
///
/// # Example
/// ```ignore
/// use vcan_fixture::cmd::{tool, CommandExt};
///
/// let output = tool!("candumpr")
///     .arg("--help")
///     .captured_output()
///     .unwrap();
/// ```
#[macro_export]
macro_rules! tool {
    ($name:literal) => {{
        let mut cmd = $crate::Command::new(env!(concat!("CARGO_BIN_EXE_", $name)));
        cmd.arg("--log-level=TRACE");
        cmd
    }};
}
