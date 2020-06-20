//! # Delays

use embedded_hal::blocking::delay::DelayMs;
use crate::core::clint::{MTIME, MTIMECMP};
use crate::clock::Clocks;
use riscv::register::{mie, mip};

/// Machine timer (mtime) as a busyloop delay provider
pub struct Delay;

impl Delay {
    /// Constructs a delay provider based on the machine timer (mtime)
    pub fn new() -> Self {
        Delay
    }
}

impl DelayMs<u32> for Delay {
    fn delay_ms(&mut self, ms: u32) {
        let ticks = (ms as u64) * 32768 / 1000;

        let mtime = MTIME;
        let t = mtime.mtime() + ticks;
        while mtime.mtime() < t { }
    }
}

impl DelayMs<u16> for Delay {
    fn delay_ms(&mut self, ms: u16) {
        self.delay_ms(u32::from(ms));
    }
}

impl DelayMs<u8> for Delay {
    fn delay_ms(&mut self, ms: u8) {
        self.delay_ms(u32::from(ms));
    }
}

/// Machine timer (mtime) as a sleep delay provider using mtimecmp
pub struct Sleep {
    clock_freq: u32,
    mtimecmp: MTIMECMP
}

impl Sleep {
    /// Constructs a delay provider using mtimecmp register to sleep
    pub fn new(mtimecmp: MTIMECMP, clocks: Clocks) -> Self {
        Sleep {
            clock_freq: clocks.lfclk().0,
            mtimecmp,
        }
    }
}

impl DelayMs<u32> for Sleep {
    fn delay_ms(&mut self, ms: u32) {
        let ticks = (ms as u64) * (self.clock_freq as u64) / 1000;
        let t = MTIME.mtime() + ticks;

        self.mtimecmp.set_mtimecmp(t);

        // Enable timer interrupt
        unsafe {
            mie::set_mtimer();
        }
        
        // Wait For Interrupt will put CPU to sleep until an interrupt hits
        // in our case when internal timer mtime value >= mtimecmp value
        // after which empty handler gets called and we go into the
        // next iteration of this loop
        loop {
            unsafe {
                riscv::asm::wfi();
            }

            // check if we got the right interrupt cause, otherwise just loop back to wfi
            if mip::read().mtimer() {
                break;
            }
        }

        // Clear timer interrupt
        unsafe {
            mie::clear_mtimer();
        }
    }
}

impl DelayMs<u16> for Sleep {
    fn delay_ms(&mut self, ms: u16) {
        self.delay_ms(u32::from(ms));
    }
}

impl DelayMs<u8> for Sleep {
    fn delay_ms(&mut self, ms: u8) {
        self.delay_ms(u32::from(ms));
    }
}

#[cfg(feature = "async-traits")]
mod async_impls {
    use core::future::Future;
    use core::pin::Pin;
    use core::task::{Context, Poll};
    use async_embedded_traits::delay::AsyncDelayMs;
    use async_embedded_traits::impl_delay_ms_for_ms_u32;
    use super::{Delay, MTIME};

    impl AsyncDelayMs<u32> for Delay {
        type DelayFuture<'f> = MtimeDelayFuture;

        fn async_delay_ms(&mut self, ms: u32) -> Self::DelayFuture<'_> {
            let ticks = (ms as u64) * 32768 / 1000;
            let mtime = MTIME;
            let deadline = mtime.mtime().wrapping_add(ticks);
            MtimeDelayFuture {
                deadline,
            }
        }
    }

    impl_delay_ms_for_ms_u32!(Delay);

    pub struct MtimeDelayFuture {
        deadline: u64,
    }

    impl Future for MtimeDelayFuture {
        type Output = ();

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
            let mtime = MTIME;
            if mtime.mtime() < self.deadline {
                cx.waker().wake_by_ref();
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        }
    }
}
