//! This module implements pinging.

use crate::packet;
use crate::sock::RawSocket;
use crate::timer::Timer;
use std::io;
use std::io::ErrorKind;
use std::io::Read;
use std::num::NonZeroU16;
use std::ptr::null_mut;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;

/// Atomic bool telling whether a `SIGALRM` signal has been received.
static ALARM: AtomicBool = AtomicBool::new(false);
/// Atomic bool telling whether a `SIGINT` signal has been received.
static INT: AtomicBool = AtomicBool::new(false);

extern "C" fn alarm_handler() {
	ALARM.store(true, Ordering::Relaxed);
}

extern "C" fn int_handler() {
	INT.store(true, Ordering::Relaxed);
}

/// A pinging context.
pub struct PingContext {
	/// The number of packets to receive.
	///
	/// If `None`, there is no limit.
	pub count: Option<NonZeroU16>,
	/// The interval between echo packets.
	pub interval: Duration,
	/// The timeout before `ping` exits regardless of how many packets have been sent.
	///
	/// If `None`, there is no deadline.
	pub deadline: Option<Duration>,
	/// The time to wait for a response for each packet.
	pub timeout: Duration,
	/// The size of packets to be sent.
	pub packet_size: usize,
	/// IP Time To Live.
	pub ttl: u8,

	/// The destination address or hostname.
	pub dest: String,

	/// The socket.
	pub sock: RawSocket,
}

impl PingContext {
	/// Sends a packet.
	///
	/// `seq` is the sequence number of the packet to send.
	fn send_packet(&mut self, seq: u16) -> io::Result<()> {
		packet::write_ping(&mut self.sock, seq, self.ttl, self.packet_size)?;
		Ok(())
	}

	/// Pings using the current context.
	///
	/// The function returns when pinging is over.
	pub fn ping(&mut self) -> io::Result<()> {
		let addr = "TODO"; // TODO resolve dns
		println!(
			"PING {} ({}) {} data bytes",
			self.dest, addr, self.packet_size
		);

		// Catch signals
		unsafe {
			libc::sigaction(
				libc::SIGALRM,
				&libc::sigaction {
					sa_sigaction: alarm_handler as _,
					sa_mask: std::mem::transmute::<_, _>([0u32; 32]),
					sa_flags: 0,
					sa_restorer: None,
				},
				null_mut::<_>(),
			);
			libc::sigaction(
				libc::SIGINT,
				&libc::sigaction {
					sa_sigaction: int_handler as _,
					sa_mask: std::mem::transmute::<_, _>([0u32; 32]),
					sa_flags: 0,
					sa_restorer: None,
				},
				null_mut::<_>(),
			);
		}

		// Start timer
		let _timer = Timer::new(self.interval);

		let start = Instant::now();

		let mut transmit_count: u16 = 0;
		let mut receive_count: u16 = 0;

		// Send first packet
		self.send_packet(transmit_count)?;
		transmit_count += 1;

		let mut buf = vec![0; u16::MAX as usize];
		let mut buf_cursor = 0;

		loop {
			// Break if count has been reached
			let cont = self.count.map(|c| receive_count < c.get()).unwrap_or(true);
			if INT.load(Ordering::Relaxed) || !cont {
				break;
			}

			// Send signal if interval has been reached
			if ALARM.load(Ordering::Relaxed) {
				// Reset timer
				ALARM.store(false, Ordering::Relaxed);

				self.send_packet(transmit_count)?;
				transmit_count += 1;
			}

			let res = self.sock.read(&mut buf[buf_cursor..]);
			match res {
				Ok(len) => buf_cursor += len,
				// If the timer expired or if pinging has been interrupted
				Err(e) if e.kind() == ErrorKind::Interrupted => continue,
				Err(e) => return Err(e),
			}

			// Check packet
			if let Some(pack) = packet::parse(&buf[..buf_cursor]) {
				// TODO
				println!(
					"{} bytes from {} ({}): icmp_seq={} ttl={} time={}",
					pack.payload_size,
					pack.src_addr,
					"TODO", // TODO
					pack.seq,
					pack.ttl,
					0 // TODO time
				);

				// Discard packet from buffer
				println!("{} {}", buf.len(), pack.size);
				buf.rotate_left(pack.size);
				println!("-> {buf_cursor}");
				buf_cursor -= pack.size;

				receive_count += 1;
			}
		}

		let elapsed = start.elapsed();

		let loss_count = if receive_count <= transmit_count {
			transmit_count - receive_count
		} else {
			0
		};
		let loss_percentage = loss_count * 100 / transmit_count;

		println!();
		println!("--- {} ping statistics ---", self.dest);
		println!(
			"{} packets transmitted, {} received, {}% packet loss, time {}ms",
			transmit_count,
			receive_count,
			loss_percentage,
			elapsed.as_millis()
		);
		// TODO end:
		// println!("rtt min/avg/max/mdev = {}/{}/{}/{} ms");

		Ok(())
	}
}
