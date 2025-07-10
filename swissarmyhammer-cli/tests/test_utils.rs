//! Test utilities for SwissArmyHammer CLI tests

/// Helper struct to ensure process cleanup in tests
///
/// This guard automatically kills and waits for a child process when dropped,
/// ensuring test processes don't leak even if a test fails or panics.
///
/// # Example
///
/// ```no_run
/// use std::process::Command;
/// use test_utils::ProcessGuard;
///
/// let child = Command::new("some_program").spawn().unwrap();
/// let _guard = ProcessGuard(child);
/// // Process will be killed when _guard goes out of scope
/// ```
pub struct ProcessGuard(pub std::process::Child);

impl Drop for ProcessGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
