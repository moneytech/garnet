// Copyright 2018 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! A futures-rs executor design specifically for Fuchsia OS.
#![feature(arbitrary_self_types, futures_api, pin)]

#![deny(warnings)]
#![deny(missing_docs)]

// Set the system allocator for anything using this crate
extern crate fuchsia_system_alloc;

/// A future which can be used by multiple threads at once.
pub mod atomic_future;

mod channel;
pub use self::channel::Channel;
mod on_signals;
pub use self::on_signals::OnSignals;
mod rwhandle;
pub use self::rwhandle::RWHandle;
mod socket;
pub use self::socket::Socket;
mod timer;
pub use self::timer::{Interval, Timer, TimeoutExt, OnTimeout};
mod executor;
pub use self::executor::{Executor, EHandle, spawn, spawn_local};
mod fifo;
pub use self::fifo::{Fifo, FifoEntry, FifoReadable, FifoWritable, ReadEntry, WriteEntry};
pub mod net;

#[macro_export]
macro_rules! many_futures {
    ($future:ident, [$first:ident, $($subfuture:ident $(,)*)*]) => {

        enum $future<$first, $($subfuture,)*> {
            $first($first),
            $(
                $subfuture($subfuture),
            )*
        }

        impl<$first, $($subfuture,)*> $crate::futures::Future for $future<$first, $($subfuture,)*>
        where
            $first: $crate::futures::Future,
            $(
                $subfuture: $crate::futures::Future<Item = $first::Item, Error = $first::Error>,
            )*
        {
            type Item = $first::Item;
            type Error = $first::Error;
            fn poll(&mut self, cx: &mut $crate::futures::task::Context) -> $crate::futures::Poll<Self::Item, Self::Error> {
                match self {
                    $future::$first(x) => $crate::futures::Future::poll(x, cx),
                    $(
                        $future::$subfuture(x) => $crate::futures::Future::poll(x, cx),
                    )*
                }
            }
        }
    }
}
