//! Backend 4: single thread, io_uring single-shot `Recv`.

use std::os::unix::io::{AsRawFd, OwnedFd};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use io_uring::{IoUring, opcode, types};

use crate::can::{CanFrame, FRAME_SIZE};

const TIMEOUT_UD: u64 = u64::MAX;

/// Receives CAN frames using io_uring with one single-shot `Recv` SQE per socket.
///
/// Each completed recv is resubmitted immediately. A periodic 100ms timeout SQE allows
/// checking the stop flag when no frames are arriving.
pub struct UringRecv {
    ring: IoUring,
    sockets: Vec<OwnedFd>,
}

impl UringRecv {
    /// Create a new receiver from non-blocking sockets (from
    /// [open_can_raw](crate::can::open_can_raw)).
    pub fn new(sockets: Vec<OwnedFd>) -> std::io::Result<Self> {
        let entries = (sockets.len() as u32 + 1).next_power_of_two().max(4);
        let ring = IoUring::new(entries)?;
        Ok(Self { ring, sockets })
    }

    /// Run until `stop` is set. Calls `on_frame` for each received frame with the socket index.
    ///
    /// Primes one `Recv` SQE per socket plus a 100ms `Timeout`. On each CQE, resubmits the
    /// corresponding operation. Returns the total number of frames received.
    pub fn run(
        &mut self,
        stop: Arc<AtomicBool>,
        on_frame: &mut dyn FnMut(usize, &CanFrame),
    ) -> std::io::Result<u64> {
        let n = self.sockets.len();
        let mut bufs = vec![[0u8; FRAME_SIZE]; n];
        let mut total = 0u64;
        let ts = types::Timespec::new().nsec(100_000_000);

        // Prime: one Recv per socket + one Timeout.
        for (idx, sock) in self.sockets.iter().enumerate() {
            let entry = opcode::Recv::new(
                types::Fd(sock.as_raw_fd()),
                bufs[idx].as_mut_ptr(),
                FRAME_SIZE as u32,
            )
            .build()
            .user_data(idx as u64);
            unsafe { self.ring.submission().push(&entry) }
                .map_err(|_| std::io::Error::other("SQ full"))?;
        }
        let timeout = opcode::Timeout::new(&ts).build().user_data(TIMEOUT_UD);
        unsafe { self.ring.submission().push(&timeout) }
            .map_err(|_| std::io::Error::other("SQ full"))?;

        while !stop.load(Ordering::Relaxed) {
            self.ring.submit_and_wait(1)?; // wait for at least 1 event

            let cqes: Vec<_> = self
                .ring
                .completion()
                .map(|cqe| (cqe.user_data(), cqe.result()))
                .collect();

            for &(ud, result) in &cqes {
                if ud == TIMEOUT_UD {
                    let entry = opcode::Timeout::new(&ts).build().user_data(TIMEOUT_UD);
                    unsafe { self.ring.submission().push(&entry) }.ok();
                    continue;
                }

                let idx = ud as usize;
                if result < 0 {
                    let err = std::io::Error::from_raw_os_error(-result);
                    if err.raw_os_error() != Some(libc::ECANCELED) {
                        return Err(err);
                    }
                } else if result == FRAME_SIZE as i32 {
                    let frame = unsafe { *(bufs[idx].as_ptr().cast::<CanFrame>()) };
                    on_frame(idx, &frame);
                    total += 1;
                }

                // Zero the buffer and resubmit Recv for this socket.
                bufs[idx].fill(0);
                let entry = opcode::Recv::new(
                    types::Fd(self.sockets[idx].as_raw_fd()),
                    bufs[idx].as_mut_ptr(),
                    FRAME_SIZE as u32,
                )
                .build()
                .user_data(ud);
                unsafe { self.ring.submission().push(&entry) }.ok();
            }
        }

        Ok(total)
    }
}
