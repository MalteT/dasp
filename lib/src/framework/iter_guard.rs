//! Guarding iterators since 2023
use std::ops::{Deref, DerefMut};

use super::Framework;

/// Simple guard to always return the extension iterator.
///
/// This will prevent iterator adapters to consume the extension iterator
/// and return said iterator to the framework upon drop.
///
/// You can just use this like any iterator, note the [`Deref`] and [`DerefMut`] implementations.
///
/// Note: If the method would normally consume this iterator, use [`Iterator::by_ref`]!
pub struct IterGuard<'f, F: Framework> {
    framework: &'f mut F,
    /// Guaranteed to be [`Some`] until dropped
    iter: Option<F::ExtensionIter>,
}

impl<'f, F: Framework> IterGuard<'f, F> {
    pub fn new(framework: &'f mut F, iter: F::ExtensionIter) -> Self {
        Self {
            framework,
            iter: Some(iter),
        }
    }
}

impl<F: Framework> Drop for IterGuard<'_, F> {
    fn drop(&mut self) {
        log::trace!("Dropping extension iter");
        let iter = self.iter.take().unwrap();
        self.framework.drop_extension_iter(iter).ok();
    }
}

impl<F: Framework> Deref for IterGuard<'_, F> {
    type Target = F::ExtensionIter;

    fn deref(&self) -> &Self::Target {
        self.iter.as_ref().unwrap()
    }
}

impl<F: Framework> DerefMut for IterGuard<'_, F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.iter.as_mut().unwrap()
    }
}
