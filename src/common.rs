use std;

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum Item<T> {
    Next(T),
    Overrun(T),
}

impl<T> Item<T> {
    pub fn into_inner(self) -> T {
        match self {
            Item::Next(x) | Item::Overrun(x) => x,
        }
    }
}

/// Error type for sending, used when the receiving end of a channel is
/// dropped
pub struct SendError<T>(pub(crate) T);

impl<T> std::fmt::Debug for SendError<T> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_tuple("SendError").field(&"...").finish()
    }
}

impl<T> std::fmt::Display for SendError<T> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "send failed because receiver is gone")
    }
}

impl<T: std::any::Any> std::error::Error for SendError<T> {
    fn description(&self) -> &str {
        "send failed because receiver is gone"
    }
}

impl<T> SendError<T> {
    /// Returns the message that was attempted to be sent but failed.
    pub fn into_inner(self) -> T {
        self.0
    }
}
