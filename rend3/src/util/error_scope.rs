//! Helpers for working with wgpu error scopes.

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use wgpu::{Device, ErrorScopeGuard};

/// Helper for working with allocation failure error scopes.
///
/// Because WebGPU uses asynchronous allocation, we cannot handle allocation failures
/// on WebGPU. This will always return success on WebGPU.


#[must_use = "All error scopes must end in a call to `end`"]
pub struct AllocationErrorScope {
    guard: Option<ErrorScopeGuard>,
    /// Used to communicate with the destructor if `end` was called on this or not.
    ended: bool,
}

impl AllocationErrorScope {
    /// Create a new AllocationErrorScope on this device.
    pub fn new(device: &Device) -> Self {
        let guard = device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
        Self { guard: Some(guard), ended: false }
    }

    pub fn end(mut self) -> Result<(), wgpu::Error> {

        self.ended = true;

        let mut future = self.guard.take().unwrap().pop();

        let pin = Pin::new(&mut future);
        match pin.poll(&mut Context::from_waker(&noop_waker::noop_waker())) {
            Poll::Ready(Some(error)) => Err(error),
            Poll::Ready(None) => Ok(()),
            Poll::Pending => Ok(()),
        }
    }
}

impl Drop for AllocationErrorScope {
    fn drop(&mut self) {
        if !self.ended {
            log::error!("AllocationErrorScope dropped without calling `end`");
        }
    }
}