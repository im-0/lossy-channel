extern crate futures;

extern crate lossy_channel;

use futures::prelude::*;
use futures::Future;
use futures::Sink;
use futures::Stream;

use futures::future::lazy;

use lossy_channel::Item;

use lossy_channel::unsync::mpsc;

#[cfg_attr(feature = "cargo-clippy", allow(needless_return))]
#[cfg_attr(feature = "cargo-clippy", allow(const_static_lifetime))]
#[cfg_attr(feature = "cargo-clippy", allow(match_same_arms))]
#[cfg_attr(feature = "cargo-clippy", allow(drop_copy))]
#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
#[cfg_attr(feature = "cargo-clippy", allow(match_wild_err_arm))]
mod support;
use support::local_executor::Core;

#[test]
fn send_recv() {
    let (tx, rx) = mpsc::channel::<i32>(1);
    let mut rx = rx.wait();

    let _ = tx.send(42).wait().unwrap();

    assert_eq!(rx.next(), Some(Ok(Item::Next(42))));
    assert_eq!(rx.next(), None);
}

#[test]
fn rx_notready() {
    let (_tx, mut rx) = mpsc::channel::<i32>(1);

    lazy(|| {
        assert_eq!(rx.poll().unwrap(), Async::NotReady);
        Ok(()) as Result<(), ()>
    }).wait()
        .unwrap();
}

#[test]
fn rx_end() {
    let (_, mut rx) = mpsc::channel::<i32>(1);

    lazy(|| {
        assert_eq!(rx.poll().unwrap(), Async::Ready(None));
        Ok(()) as Result<(), ()>
    }).wait()
        .unwrap();
}

#[test]
fn tx_err() {
    let (tx, _) = mpsc::channel::<i32>(1);
    lazy(move || {
        assert!(tx.send(2).poll().is_err());
        Ok(()) as Result<(), ()>
    }).wait()
        .unwrap();
}

#[test]
fn recv_unpark() {
    let core = Core::new();
    let (tx, rx) = mpsc::channel::<i32>(1);
    let tx2 = tx.clone();
    core.spawn(
        rx.map(|x| x.into_inner())
            .collect()
            .map(|xs| assert_eq!(xs, [1, 2])),
    );
    core.spawn(lazy(move || {
        tx.send(1).map(|_| ()).map_err(|e| panic!("{}", e))
    }));
    core.run(lazy(move || {
        tx2.send(2)
    })).unwrap();
}
