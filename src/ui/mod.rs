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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_prompt_cancellation_ok() {
        let result = handle_prompt_cancellation(|| Ok(()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_prompt_cancellation_operation_canceled() {
        let result = handle_prompt_cancellation(|| Err(InquireError::OperationCanceled.into()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_prompt_cancellation_operation_interrupted() {
        let result = handle_prompt_cancellation(|| Err(InquireError::OperationInterrupted.into()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_prompt_cancellation_other_error() {
        let result = handle_prompt_cancellation(|| Err(anyhow::anyhow!("Some other error")));
        let Err(err) = result else {
            panic!("expected an error");
        };
        assert!(err.to_string().contains("Some other error"));
    }

    #[test]
    fn test_is_prompt_cancelled_operation_canceled() {
        assert!(is_prompt_cancelled(&InquireError::OperationCanceled));
    }

    #[test]
    fn test_is_prompt_cancelled_operation_interrupted() {
        assert!(is_prompt_cancelled(&InquireError::OperationInterrupted));
    }

    #[test]
    fn test_is_prompt_cancelled_other_error() {
        let err = InquireError::Custom("test".into());
        assert!(!is_prompt_cancelled(&err));
    }
}
