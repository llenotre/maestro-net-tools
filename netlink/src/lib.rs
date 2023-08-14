//! Netlink is a kernel feature allowing to inspect and control network interfaces.
//!
//! The netlink interface can be accessed from userspace through a socket.

pub mod route;
pub mod util;

use std::ffi::*;
use std::io;
use std::marker::PhantomData;

/// Netlink family: route
const NETLINK_ROUTE: c_int = 0;

/// Netlink message header.
#[repr(C)]
struct NlMsgHdr {
	/// Length of the message including header
	nlmsg_len: u32,
	/// Type of message content
	nlmsg_type: u16,
	/// Additional flags
	nlmsg_flags: u16,
	/// Sequence number
	nlmsg_seq: u32,
	/// Sender port ID
	nlmsg_pid: u32,
}

/// A netlink socket.
pub struct Netlink {
	/// The socket's file descriptor.
	fd: c_int,
}

impl Netlink {
	/// Creates a new instance.
	///
	/// `family` is the netlink group to communicate with.
	pub fn new(family: c_int) -> io::Result<Self> {
		let fd = unsafe { libc::socket(libc::AF_NETLINK, libc::SOCK_RAW, family) };
		if fd < 0 {
			return Err(io::Error::last_os_error());
		}

		Ok(Self {
			fd,
		})
	}

	/// Low-level interface to send messages on the socket.
	pub unsafe fn send_to(&self, _buf: &[u8]) -> io::Result<()> {
		// TODO
		todo!()
	}
}

impl Drop for Netlink {
	fn drop(&mut self) {
		unsafe {
			libc::close(self.fd);
		}
	}
}

/// An iterator on netlink objects.
pub struct NetlinkIter<'sock, T> {
	/// The netlink socket.
	sock: &'sock Netlink,
	/// The sequence number on which the iterator works.
	seq: u32,

	_phantom: PhantomData<T>,
}

impl<'sock, T> Iterator for NetlinkIter<'sock, T> {
	type Item = io::Result<T>;

	fn next(&mut self) -> Option<Self::Item> {
		// TODO
		todo!()
	}
}