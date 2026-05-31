//! Message passing helpers for the UI
// (c) 2026 Ross Younger

use std::sync::mpsc::{Receiver, Sender, channel};

/// A convenience wrapper to the `std::sync::mpsc` suite.
#[derive(Debug)]
pub struct Channel<T> {
    sender: Sender<T>,
    receiver: Receiver<T>,
}

impl<T> Channel<T> {
    /// Accessor that clones the sender
    #[must_use]
    pub fn sender(&self) -> Sender<T> {
        self.sender.clone()
    }

    /// Try to receive a message without blocking. Returns `None` if no message is available,
    /// or if the channel is closed.
    /// (We don't drop the sender for as long as this struct lives, so we don't need to worry about
    /// the `Disconnected` result unless you're being especially devious.)
    #[must_use]
    pub fn try_recv(&self) -> Option<T> {
        self.receiver.try_recv().ok()
    }

    /// Blocking wait for a message from the channel
    ///
    /// # Panics
    /// If the sender half has become closed (which requires you to be quite devious)
    #[must_use]
    pub fn recv(&self) -> T {
        self.receiver.recv().unwrap()
    }
}

impl<T> Default for Channel<T> {
    fn default() -> Self {
        let (sender, receiver) = channel();
        Self { sender, receiver }
    }
}

/// Try to receive a message from the channel and, if successful, update the given field with it.
/// This is intended to be used in the UI main loop to update state from messages sent by other
/// threads.
pub fn update_field_from_channel<T, U>(channel: &Channel<T>, field: &mut Option<U>)
where
    T: Into<U>,
{
    if let Some(i) = channel.try_recv() {
        *field = Some(i.into());
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn channel() {
        let channel = Channel::<i32>::default();
        let sender = channel.sender();
        assert_eq!(channel.try_recv(), None);
        sender.send(42).unwrap();
        sender.send(57).unwrap();
        assert_eq!(channel.try_recv().unwrap(), 42);
        assert_eq!(channel.recv(), 57);
        assert_eq!(channel.try_recv(), None);
    }

    #[test]
    fn update_field_from_channel_with_message() {
        let channel: Channel<i32> = Channel::default();
        let sender = channel.sender();
        let mut field: Option<i32> = None;

        sender.send(42).unwrap();
        update_field_from_channel(&channel, &mut field);

        assert_eq!(field, Some(42));
    }

    #[test]
    fn update_field_from_channel_no_message() {
        let channel: Channel<i32> = Channel::default();
        let mut field: Option<i32> = None;

        update_field_from_channel(&channel, &mut field);

        assert_eq!(field, None);
    }

    #[test]
    fn update_field_from_channel_overwrites_existing() {
        let channel: Channel<i32> = Channel::default();
        let sender = channel.sender();
        let mut field: Option<i32> = Some(10);

        sender.send(42).unwrap();
        update_field_from_channel(&channel, &mut field);

        assert_eq!(field, Some(42));
    }

    #[test]
    fn update_field_from_channel_with_conversion() {
        let channel: Channel<i32> = Channel::default();
        let sender = channel.sender();
        let mut field: Option<i64> = None;

        sender.send(42i32).unwrap();
        update_field_from_channel(&channel, &mut field);

        assert_eq!(field, Some(42i64));
    }
}
