use std;

use futures;

use super::super::common;

type Deque<T> = std::collections::VecDeque<T>;

#[derive(Debug)]
struct Shared<T> {
    buffer: Deque<T>,
    capacity: usize,
    blocked_recv: Option<futures::task::Task>,
    overrun: bool,
}

#[derive(Debug)]
pub struct Sender<T> {
    shared: std::rc::Weak<std::cell::RefCell<Shared<T>>>,
}

#[derive(Debug)]
pub struct Receiver<T> {
    shared: std::rc::Rc<std::cell::RefCell<Shared<T>>>,
}

pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    assert!(capacity > 0, "channel capacity should be >0");

    let shared = std::rc::Rc::new(std::cell::RefCell::new(Shared {
        buffer: std::collections::VecDeque::with_capacity(capacity),
        capacity,
        blocked_recv: None,
        overrun: false,
    }));
    let sender = Sender {
        shared: std::rc::Rc::downgrade(&shared),
    };
    let receiver = Receiver { shared };

    (sender, receiver)
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            shared: std::rc::Weak::clone(&self.shared),
        }
    }
}

impl<T> futures::Sink for Sender<T> {
    type SinkItem = T;
    type SinkError = common::SendError<T>;

    fn start_send(&mut self, msg: Self::SinkItem) -> futures::StartSend<Self::SinkItem, Self::SinkError> {
        let shared = match self.shared.upgrade() {
            Some(shared) => shared,
            None => return Err(common::SendError(msg)),
        };
        let mut shared = shared.borrow_mut();

        if shared.buffer.len() == shared.capacity {
            shared.overrun = true;
            let _ = shared.buffer.pop_front();
        }
        shared.buffer.push_back(msg);

        if let Some(task) = shared.blocked_recv.take() {
            drop(shared);
            task.notify();
        }

        Ok(futures::AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> futures::Poll<(), Self::SinkError> {
        Ok(futures::Async::Ready(()))
    }

    fn close(&mut self) -> futures::Poll<(), Self::SinkError> {
        Ok(futures::Async::Ready(()))
    }
}

impl<T> Receiver<T> {
    fn pop_front(shared: &mut Shared<T>) -> Option<common::Item<T>> {
        shared.buffer.pop_front().map(|msg| {
            if shared.overrun {
                shared.overrun = false;
                common::Item::Overrun(msg)
            } else {
                common::Item::Next(msg)
            }
        })
    }
}

impl<T> futures::Stream for Receiver<T> {
    type Item = common::Item<T>;
    type Error = ();

    fn poll(&mut self) -> futures::Poll<Option<Self::Item>, Self::Error> {
        if let Some(shared) = std::rc::Rc::get_mut(&mut self.shared) {
            // All senders have been dropped, so drain the buffer and end the
            // stream.
            return Ok(futures::Async::Ready(Self::pop_front(&mut shared
                .borrow_mut())));
        }

        let mut shared = self.shared.borrow_mut();
        if let Some(msg) = Self::pop_front(&mut shared) {
            Ok(futures::Async::Ready(Some(msg)))
        } else {
            shared.blocked_recv = Some(futures::task::current());
            Ok(futures::Async::NotReady)
        }
    }
}
