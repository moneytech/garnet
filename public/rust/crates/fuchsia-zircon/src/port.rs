// Copyright 2017 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Type-safe bindings for Zircon port objects.

use std::mem;

use {AsHandleRef, HandleBased, Handle, HandleRef, Signals, Status, Time};
use {sys, ok};

/// An object representing a Zircon
/// [port](https://fuchsia.googlesource.com/zircon/+/master/docs/objects/port.md).
///
/// As essentially a subtype of `Handle`, it can be freely interconverted.
#[derive(Debug, Eq, PartialEq)]
pub struct Port(Handle);
impl_handle_based!(Port);

/// A packet sent through a port. This is a type-safe wrapper for
/// [zx_port_packet_t](https://fuchsia.googlesource.com/zircon/+/master/docs/syscalls/port_wait.md).
#[derive(PartialEq, Eq, Debug)]
pub struct Packet(sys::zx_port_packet_t);

/// The contents of a `Packet`.
#[derive(Debug, Copy, Clone)]
pub enum PacketContents {
    /// A user-generated packet.
    User(UserPacket),
    /// A one-shot signal packet generated via `object_wait_async`.
    SignalOne(SignalPacket),
    /// A repeating signal packet generated via `object_wait_async`.
    SignalRep(SignalPacket),

    #[doc(hidden)]
    __Nonexhaustive
}

/// Contents of a user packet (one sent by `port_queue`). This is a type-safe wrapper for
/// [zx_packet_user_t](https://fuchsia.googlesource.com/zircon/+/master/docs/syscalls/port_wait.md).
#[derive(Debug, Copy, Clone)]
pub struct UserPacket(sys::zx_packet_user_t);

/// Contents of a signal packet (one generated by the kernel). This is a type-safe wrapper for
/// [zx_packet_signal_t](https://fuchsia.googlesource.com/zircon/+/master/docs/syscalls/port_wait.md).
#[derive(Debug, Copy, Clone)]
pub struct SignalPacket(sys::zx_packet_signal_t);

impl Packet {
    /// Creates a new packet with `UserPacket` data.
    pub fn from_user_packet(key: u64, status: i32, user: UserPacket) -> Packet {
        Packet(
            sys::zx_port_packet_t {
                key: key,
                packet_type: sys::zx_packet_type_t::ZX_PKT_TYPE_USER,
                status: status,
                union: user.0,
            }
        )
    }

    /// The packet's key.
    pub fn key(&self) -> u64 {
        self.0.key
    }

    /// The packet's status.
    // TODO: should this type be wrapped?
    pub fn status(&self) -> i32 {
        self.0.status
    }

    /// The contents of the packet.
    pub fn contents(&self) -> PacketContents {
        if self.0.packet_type == sys::zx_packet_type_t::ZX_PKT_TYPE_USER {
            PacketContents::User(UserPacket(self.0.union))
        } else if self.0.packet_type == sys::zx_packet_type_t::ZX_PKT_TYPE_SIGNAL_ONE {
            PacketContents::SignalOne(SignalPacket(unsafe { mem::transmute_copy(&self.0.union) }))
        } else if self.0.packet_type == sys::zx_packet_type_t::ZX_PKT_TYPE_SIGNAL_REP {
            PacketContents::SignalRep(SignalPacket(unsafe { mem::transmute_copy(&self.0.union) }))
        } else {
            panic!("unexpected packet type");
        }
    }
}

impl UserPacket {
    pub fn from_u8_array(val: [u8; 32]) -> UserPacket {
        UserPacket(val)
    }

    pub fn as_u8_array(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn as_mut_u8_array(&mut self) -> &mut [u8; 32] {
        &mut self.0
    }
}

impl SignalPacket {
    /// The signals used in the call to `object_wait_async`.
    pub fn trigger(&self) -> Signals {
        Signals::from_bits_truncate(self.0.trigger)
    }

    /// The observed signals.
    pub fn observed(&self) -> Signals {
        Signals::from_bits_truncate(self.0.observed)
    }

    /// A per object count of pending operations.
    pub fn count(&self) -> u64 {
        self.0.count
    }
}

impl Port {
    /// Create an IO port, allowing IO packets to be read and enqueued.
    ///
    /// Wraps the
    /// [zx_port_create](https://fuchsia.googlesource.com/zircon/+/master/docs/syscalls/port_create.md)
    /// syscall.
    pub fn create() -> Result<Port, Status> {
        unsafe {
            let mut handle = 0;
            let opts = 0;
            let status = sys::zx_port_create(opts, &mut handle);
            ok(status)?;
            Ok(Handle::from_raw(handle).into())
        }
    }

    /// Attempt to queue a user packet to the IO port.
    ///
    /// Wraps the
    /// [zx_port_queue](https://fuchsia.googlesource.com/zircon/+/master/docs/syscalls/port_queue.md)
    /// syscall.
    pub fn queue(&self, packet: &Packet) -> Result<(), Status> {
        let status = unsafe {
            sys::zx_port_queue(self.raw_handle(),
                &packet.0 as *const sys::zx_port_packet_t, 1)
        };
        ok(status)
    }

    /// Wait for a packet to arrive on a (V2) port.
    ///
    /// Wraps the
    /// [zx_port_wait](https://fuchsia.googlesource.com/zircon/+/master/docs/syscalls/port_wait.md)
    /// syscall.
    pub fn wait(&self, deadline: Time) -> Result<Packet, Status> {
        let mut packet = Default::default();
        let status = unsafe {
            sys::zx_port_wait(self.raw_handle(), deadline.nanos(),
                &mut packet as *mut sys::zx_port_packet_t, 1)
        };
        ok(status)?;
        Ok(Packet(packet))
    }

    /// Cancel pending wait_async calls for an object with the given key.
    ///
    /// Wraps the
    /// [zx_port_cancel](https://fuchsia.googlesource.com/zircon/+/HEAD/docs/syscalls/port_cancel.md)
    /// syscall.
    pub fn cancel<H>(&self, source: &H, key: u64) -> Result<(), Status> where H: HandleBased {
        let status = unsafe {
            sys::zx_port_cancel(self.raw_handle(), source.raw_handle(), key)
        };
        ok(status)
    }
}

/// Options for wait_async.
#[repr(u32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum WaitAsyncOpts {
    Once = sys::ZX_WAIT_ASYNC_ONCE,
    Repeating = sys::ZX_WAIT_ASYNC_REPEATING,
}

#[cfg(test)]
mod tests {
    use super::*;
    use {DurationNum, Event};

    #[test]
    fn port_basic() {
        let ten_ms = 10.millis();

        let port = Port::create().unwrap();

        // Waiting now should time out.
        assert_eq!(port.wait(ten_ms.after_now()), Err(Status::TIMED_OUT));

        // Send a valid packet.
        let packet = Packet::from_user_packet(
            42,
            123,
            UserPacket::from_u8_array([13; 32]),
        );
        assert!(port.queue(&packet).is_ok());

        // Waiting should succeed this time. We should get back the packet we sent.
        let read_packet = port.wait(ten_ms.after_now()).unwrap();
        assert_eq!(read_packet, packet);
    }

    #[test]
    fn wait_async_once() {
        let ten_ms = 10.millis();
        let key = 42;

        let port = Port::create().unwrap();
        let event = Event::create().unwrap();

        assert!(event.wait_async_handle(&port, key, Signals::USER_0 | Signals::USER_1,
            WaitAsyncOpts::Once).is_ok());

        // Waiting without setting any signal should time out.
        assert_eq!(port.wait(ten_ms.after_now()), Err(Status::TIMED_OUT));

        // If we set a signal, we should be able to wait for it.
        assert!(event.signal_handle(Signals::NONE, Signals::USER_0).is_ok());
        let read_packet = port.wait(ten_ms.after_now()).unwrap();
        assert_eq!(read_packet.key(), key);
        assert_eq!(read_packet.status(), 0);
        match read_packet.contents() {
            PacketContents::SignalOne(sig) => {
                assert_eq!(sig.trigger(), Signals::USER_0 | Signals::USER_1);
                assert_eq!(sig.observed(), Signals::USER_0);
                assert_eq!(sig.count(), 1);
            }
            _ => panic!("wrong packet type"),
        }

        // Shouldn't get any more packets.
        assert_eq!(port.wait(ten_ms.after_now()), Err(Status::TIMED_OUT));

        // Calling wait_async again should result in another packet.
        assert!(event.wait_async_handle(&port, key, Signals::USER_0, WaitAsyncOpts::Once).is_ok());
        let read_packet = port.wait(ten_ms.after_now()).unwrap();
        assert_eq!(read_packet.key(), key);
        assert_eq!(read_packet.status(), 0);
        match read_packet.contents() {
            PacketContents::SignalOne(sig) => {
                assert_eq!(sig.trigger(), Signals::USER_0);
                assert_eq!(sig.observed(), Signals::USER_0);
                assert_eq!(sig.count(), 1);
            }
            _ => panic!("wrong packet type"),
        }

        // Calling wait_async_handle then cancel, we should not get a packet as cancel will
        // remove it from  the queue.
        assert!(event.wait_async_handle(&port, key, Signals::USER_0, WaitAsyncOpts::Once).is_ok());
        assert!(port.cancel(&event, key).is_ok());
        assert_eq!(port.wait(ten_ms.after_now()), Err(Status::TIMED_OUT));

        // If the event is signalled after the cancel, we also shouldn't get a packet.
        assert!(event.signal_handle(Signals::USER_0, Signals::NONE).is_ok());  // clear signal
        assert!(event.wait_async_handle(&port, key, Signals::USER_0, WaitAsyncOpts::Once).is_ok());
        assert!(port.cancel(&event, key).is_ok());
        assert!(event.signal_handle(Signals::NONE, Signals::USER_0).is_ok());
        assert_eq!(port.wait(ten_ms.after_now()), Err(Status::TIMED_OUT));
    }

    #[test]
    fn wait_async_repeating() {
        let ten_ms = 10.millis();
        let key = 42;

        let port = Port::create().unwrap();
        let event = Event::create().unwrap();

        assert!(event.wait_async_handle(&port, key, Signals::USER_0 | Signals::USER_1,
            WaitAsyncOpts::Repeating).is_ok());

        // Waiting without setting any signal should time out.
        assert_eq!(port.wait(ten_ms.after_now()), Err(Status::TIMED_OUT));

        // If we set a signal, we should be able to wait for it.
        assert!(event.signal_handle(Signals::NONE, Signals::USER_0).is_ok());
        let read_packet = port.wait(ten_ms.after_now()).unwrap();
        assert_eq!(read_packet.key(), key);
        assert_eq!(read_packet.status(), 0);
        match read_packet.contents() {
            PacketContents::SignalRep(sig) => {
                assert_eq!(sig.trigger(), Signals::USER_0 | Signals::USER_1);
                assert_eq!(sig.observed(), Signals::USER_0);
                assert_eq!(sig.count(), 1);
            }
            _ => panic!("wrong packet type"),
        }

        // Should not get any more packets, as ZX_WAIT_ASYNC_REPEATING is edge triggered rather than
        // level triggered.
        assert_eq!(port.wait(ten_ms.after_now()), Err(Status::TIMED_OUT));

        // If we clear and resignal, we should get the same packet again,
        // even though we didn't call event.wait_async again.
        assert!(event.signal_handle(Signals::USER_0, Signals::NONE).is_ok());  // clear signal
        assert!(event.signal_handle(Signals::NONE, Signals::USER_0).is_ok());
        let read_packet = port.wait(ten_ms.after_now()).unwrap();
        assert_eq!(read_packet.key(), key);
        assert_eq!(read_packet.status(), 0);
        match read_packet.contents() {
            PacketContents::SignalRep(sig) => {
                assert_eq!(sig.trigger(), Signals::USER_0 | Signals::USER_1);
                assert_eq!(sig.observed(), Signals::USER_0);
                assert_eq!(sig.count(), 1);
            }
            _ => panic!("wrong packet type"),
        }

        // Cancelling the wait should stop us getting packets...
        assert!(port.cancel(&event, key).is_ok());
        assert_eq!(port.wait(ten_ms.after_now()), Err(Status::TIMED_OUT));
        // ... even if we clear and resignal
        assert!(event.signal_handle(Signals::USER_0, Signals::NONE).is_ok());  // clear signal
        assert!(event.signal_handle(Signals::NONE, Signals::USER_0).is_ok());
        assert_eq!(port.wait(ten_ms.after_now()), Err(Status::TIMED_OUT));

        // Calling wait_async again should result in another packet.
        assert!(event.wait_async_handle(
            &port, key, Signals::USER_0, WaitAsyncOpts::Repeating).is_ok());
        let read_packet = port.wait(ten_ms.after_now()).unwrap();
        assert_eq!(read_packet.key(), key);
        assert_eq!(read_packet.status(), 0);
        match read_packet.contents() {
            PacketContents::SignalRep(sig) => {
                assert_eq!(sig.trigger(), Signals::USER_0);
                assert_eq!(sig.observed(), Signals::USER_0);
                assert_eq!(sig.count(), 1);
            }
            _ => panic!("wrong packet type"),
        }

        // Closing the handle should stop us getting packets.
        drop(event);
        assert_eq!(port.wait(ten_ms.after_now()), Err(Status::TIMED_OUT));
    }
}
