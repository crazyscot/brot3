// Inner mutability helper class
// (c) 2024 Ross Younger

use tauri::async_runtime::RwLock;

#[derive(Default)]
pub struct InnerMutable<T: Clone> {
    inner: RwLock<T>,
}

#[allow(dead_code)]

impl<T: Clone> InnerMutable<T> {
    /// Clones the inner contents, blocking
    pub fn clone_blocking(&self) -> T {
        let guard = self.inner.blocking_read();
        guard.clone()
    }

    /// Replaces the inner contents, blocking
    pub fn replace_blocking(&self, replacement: &T) {
        let mut guard = self.inner.blocking_write();
        (*guard).clone_from(replacement);
    }

    /// Clones the inner contents, async
    pub async fn clone_async(&self) -> T {
        (*self.inner.read().await).clone()
    }

    /// Replaces the inner contents, async
    pub async fn replace_async(&self, replacement: &T) {
        (*self.inner.write().await).clone_from(replacement)
    }
}

#[cfg(test)]
mod tests {
    use crate::mutable_util::InnerMutable;

    #[derive(Default, Clone)]
    struct Tester {
        foo: i32,
    }

    #[test]
    fn replace_blocking() {
        let uut = InnerMutable::<Tester>::default();
        let def = uut.clone_blocking();

        let repl = Tester { foo: 42 };
        assert_ne!(def.foo, repl.foo);
        uut.replace_blocking(&repl);

        let result = uut.clone_blocking();
        assert_eq!(result.foo, repl.foo);
    }

    // sadly, test functions can't be async at the moment, so we'll need to go via tokio
    #[tokio_macros::test]
    async fn replace_async() {
        let uut = InnerMutable::<Tester>::default();
        let def = uut.clone_async().await;

        let repl = Tester { foo: 42 };
        assert_ne!(def.foo, repl.foo);
        uut.replace_async(&repl).await;

        let result = uut.clone_async().await;
        assert_eq!(result.foo, repl.foo);
    }
}
