use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A wrapped future that provides a sync interface to async operations
pub struct WrappedFuture<T> {
    inner: Pin<Box<dyn Future<Output = T> + Send + 'static>>,
}

impl<T> WrappedFuture<T> {
    pub fn new<F>(future: F) -> Self
    where
        F: Future<Output = T> + Send + 'static,
    {
        Self {
            inner: Box::pin(future),
        }
    }
}

impl<T> Future for WrappedFuture<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.inner.as_mut().poll(cx)
    }
}
