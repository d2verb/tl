use anyhow::Result;
use inquire::InquireError;

mod spinner;
mod style;

pub use spinner::Spinner;
pub use style::Style;

/// Check if the inquire error is a user cancellation/interruption.
const fn is_prompt_cancelled(err: &InquireError) -> bool {
    matches!(
        err,
        InquireError::OperationCanceled | InquireError::OperationInterrupted
    )
}

/// Wraps a function that uses interactive prompts and handles user cancellation gracefully.
///
/// If the user cancels the prompt (Ctrl+C or Escape), this function prints a newline
/// to clean up the terminal and returns `Ok(())` instead of propagating the error.
pub fn handle_prompt_cancellation<F>(f: F) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    match f() {
        Ok(()) => Ok(()),
        Err(e)
            if e.downcast_ref::<InquireError>()
                .is_some_and(is_prompt_cancelled) =>
        {
            println!();
            Ok(())
        }
        Err(e) => Err(e),
    }
}
