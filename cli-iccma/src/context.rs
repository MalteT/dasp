use lib::framework::{Framework, IterGuard};

use crate::{args::Args, Result};

/// Context for task execution.
///
/// Not that useful yet.
pub struct Context<F: Framework> {
    /// Framework we're handling.
    framework: F,
}

impl<F: Framework> Context<F> {
    pub fn new(args: &Args) -> Result<Self> {
        // Parse given input file
        debug_assert!(
            args.file_content()?.is_some(),
            "File expected but not found"
        );
        let content = args.file_content()?.unwrap();
        Ok(Context {
            framework: F::new(&content)?,
        })
    }

    pub fn count_extensions(&mut self) -> Result<usize> {
        self.framework.count_extensions()
    }

    pub fn update(&mut self, update_line: &str) -> Result<()> {
        self.framework.update(update_line)
    }

    pub fn sample_extension(&mut self) -> Result<Option<F::Extension>> {
        self.framework.sample_extension()
    }

    pub fn enumerate_extensions(&mut self) -> Result<IterGuard<F>> {
        self.framework.enumerate_extensions()
    }
}
