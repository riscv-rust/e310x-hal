//! # Serial Peripheral Interface
//!
//! You can use the `Spi` interface with these SPI instances
//!
//! # QSPI0
//! - Interrupt::QSPI0
//!
//! # QSPI1
//! - MOSI: Pin 3 IOF0
//! - MISO: Pin 4 IOF0
//! - SCK: Pin 5 IOF0
//! - SS0: Pin 2 IOF0
//! - SS1: Pin 8 IOF0 (not connected to package in FE310)
//! - SS2: Pin 9 IOF0
//! - SS3: Pin 10 IOF0
//! - Interrupt::QSPI1
//!
//! # QSPI2
//! *Warning:* QSPI2 pins are not connected to package in FE310
//! - MOSI: Pin 27 IOF0
//! - MISO: Pin 28 IOF0
//! - SCK: Pin 29 IOF0
//! - SS: Pin 26 IOF0
//! - Interrupt::QSPI2

use core::convert::Infallible;
use core::ops::Deref;
pub use embedded_hal::spi::{Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3};
use e310x::{QSPI0, QSPI1, QSPI2, qspi0};
use crate::clock::Clocks;
use crate::time::Hertz;
use nb;


/// SPI pins - DO NOT IMPLEMENT THIS TRAIT
///
/// This trait is implemented for pin tuples (), (MOSI, MISO, SCK) and (MOSI, MISO, SCK, SS)
/// and combinations without MOSI/MISO
pub trait Pins<SPI> {
    #[doc(hidden)]
    const CS_INDEX: Option<u32>;
}

/* SPI0 pins */
impl Pins<QSPI0> for () {
    const CS_INDEX: Option<u32> = Some(0);
}

/* SPI1 pins */
mod spi1_impl {
    use crate::gpio::{NoInvert, IOF0};
    use crate::gpio::gpio0;
    use super::{Pins, QSPI1};

    type MOSI = gpio0::Pin3<IOF0<NoInvert>>;
    type MISO = gpio0::Pin4<IOF0<NoInvert>>;
    type SCK = gpio0::Pin5<IOF0<NoInvert>>;
    type SS0 = gpio0::Pin2<IOF0<NoInvert>>;
    type SS1 = gpio0::Pin8<IOF0<NoInvert>>;
    type SS2 = gpio0::Pin9<IOF0<NoInvert>>;
    type SS3 = gpio0::Pin10<IOF0<NoInvert>>;

    impl Pins<QSPI1> for (MOSI, MISO, SCK) { const CS_INDEX: Option<u32> = None; }
    impl Pins<QSPI1> for (MOSI, (),   SCK) { const CS_INDEX: Option<u32> = None; }
    impl Pins<QSPI1> for ((),   MISO, SCK) { const CS_INDEX: Option<u32> = None; }
    impl Pins<QSPI1> for (MOSI, MISO, SCK, SS0) { const CS_INDEX: Option<u32> = Some(0); }
    impl Pins<QSPI1> for (MOSI, (),   SCK, SS0) { const CS_INDEX: Option<u32> = Some(0); }
    impl Pins<QSPI1> for ((),   MISO, SCK, SS0) { const CS_INDEX: Option<u32> = Some(0); }
    impl Pins<QSPI1> for (MOSI, MISO, SCK, SS1) { const CS_INDEX: Option<u32> = Some(1); }
    impl Pins<QSPI1> for (MOSI, (),   SCK, SS1) { const CS_INDEX: Option<u32> = Some(1); }
    impl Pins<QSPI1> for ((),   MISO, SCK, SS1) { const CS_INDEX: Option<u32> = Some(1); }
    impl Pins<QSPI1> for (MOSI, MISO, SCK, SS2) { const CS_INDEX: Option<u32> = Some(2); }
    impl Pins<QSPI1> for (MOSI, (),   SCK, SS2) { const CS_INDEX: Option<u32> = Some(2); }
    impl Pins<QSPI1> for ((),   MISO, SCK, SS2) { const CS_INDEX: Option<u32> = Some(2); }
    impl Pins<QSPI1> for (MOSI, MISO, SCK, SS3) { const CS_INDEX: Option<u32> = Some(3); }
    impl Pins<QSPI1> for (MOSI, (),   SCK, SS3) { const CS_INDEX: Option<u32> = Some(3); }
    impl Pins<QSPI1> for ((),   MISO, SCK, SS3) { const CS_INDEX: Option<u32> = Some(3); }
}

/* SPI2 pins */
mod spi2_impl {
    use crate::gpio::{NoInvert, IOF0};
    use crate::gpio::gpio0;
    use super::{Pins, QSPI2};

    type MOSI = gpio0::Pin27<IOF0<NoInvert>>;
    type MISO = gpio0::Pin28<IOF0<NoInvert>>;
    type SCK = gpio0::Pin29<IOF0<NoInvert>>;
    type SS0 = gpio0::Pin26<IOF0<NoInvert>>;

    impl Pins<QSPI2> for (MOSI, MISO, SCK) { const CS_INDEX: Option<u32> = None; }
    impl Pins<QSPI2> for (MOSI, (),   SCK) { const CS_INDEX: Option<u32> = None; }
    impl Pins<QSPI2> for ((),   MISO, SCK) { const CS_INDEX: Option<u32> = None; }
    impl Pins<QSPI2> for (MOSI, MISO, SCK, SS0) { const CS_INDEX: Option<u32> = Some(0); }
    impl Pins<QSPI2> for (MOSI, (),   SCK, SS0) { const CS_INDEX: Option<u32> = Some(0); }
    impl Pins<QSPI2> for ((),   MISO, SCK, SS0) { const CS_INDEX: Option<u32> = Some(0); }
}


#[doc(hidden)]
pub trait SpiX: Deref<Target = qspi0::RegisterBlock> {}
impl SpiX for QSPI0 {}
impl SpiX for QSPI1 {}
impl SpiX for QSPI2 {}


/// SPI abstraction
pub struct Spi<SPI, PINS> {
    spi: SPI,
    pins: PINS,
}

impl<SPI: SpiX, PINS> Spi<SPI, PINS> {
    /// Configures the SPI peripheral to operate in full duplex master mode
    /// Defaults to using AUTO CS in FRAME mode if PINS configuration allows it
    pub fn new(spi: SPI, pins: PINS, mode: Mode, freq: Hertz, clocks: Clocks) -> Self
    where
        PINS: Pins<SPI>
    {
        let div = clocks.tlclk().0 / (2 * freq.0) - 1;
        spi.div.write(|w| unsafe { w.bits(div) });

        let cs_mode = if let Some(cs_index) = PINS::CS_INDEX {
            spi.csid.write(|w| unsafe { w.bits(cs_index) });

            2 // AUTO: Assert/de-assert CS at the beginning/end of each frame
        } else {
            3 // OFF: Disable hardware control of the CS pin
        };
        spi.csmode.write(|w| unsafe { w.bits(cs_mode) });

        // Set CS pin polarity to high
        spi.csdef.reset();

        // Set SPI mode
        let phase = mode.phase == Phase::CaptureOnSecondTransition;
        let polarity = mode.polarity == Polarity::IdleHigh;
        spi.mode.write(|w| w
            .phase().bit(phase)
            .polarity().bit(polarity)
        );

        spi.fmt.write(|w| unsafe { w
            .protocol().bits(0) // Single
            .endian().clear_bit() // Transmit most-significant bit (MSB) first
            .direction().rx()
            .length().bits(8)
        });

        // Set watermark levels
        spi.txmark.write(|w| unsafe { w.value().bits(1) });
        spi.rxmark.write(|w| unsafe { w.value().bits(0) });

        spi.delay0.reset();
        spi.delay1.reset();

        Self { spi, pins }
    }

    /// Sets transmit watermark level
    pub fn set_tx_watermark(&mut self, value: u8) {
        self.spi.txmark.write(|w| unsafe { w.value().bits(value) });
    }

    /// Sets receive watermark level
    pub fn set_rx_watermark(&mut self, value: u8) {
        self.spi.rxmark.write(|w| unsafe { w.value().bits(value) });
    }

    /// Returns transmit watermark event status
    pub fn tx_wm_is_pending(&self) -> bool {
        self.spi.ip.read().txwm().bit()
    }

    /// Returns receive watermark event status
    pub fn rx_wm_is_pending(&self) -> bool {
        self.spi.ip.read().rxwm().bit()
    }

    /// Starts listening for transmit watermark interrupt event
    pub fn listen_tx_wm(&mut self) {
        self.spi.ie.write(|w| w.txwm().set_bit())
    }

    /// Starts listening for receive watermark interrupt event
    pub fn listen_rx_wm(&mut self) {
        self.spi.ie.write(|w| w.rxwm().set_bit())
    }

    /// Stops listening for transmit watermark interrupt event
    pub fn unlisten_tx_wm(&mut self) {
        self.spi.ie.write(|w| w.txwm().clear_bit())
    }

    /// Stops listening for receive watermark interrupt event
    pub fn unlisten_rx_wm(&mut self) {
        self.spi.ie.write(|w| w.rxwm().clear_bit())
    }

    /// Set AUTO CS mode to per-word operation
    pub fn cs_mode_word(&mut self) {
        if self.spi.csmode.read().bits() != 3 {
            self.spi.csmode.write(|w| unsafe { w.bits(0) });
        }
    }

    /// Set AUTO CS mode to per-frame operation
    pub fn cs_mode_frame(&mut self) {
        if self.spi.csmode.read().bits() != 3 {
            self.spi.csmode.write(|w| unsafe { w.bits(2) });
        }
    }

    /// Finishes transfer by deasserting CS (only for hardware-controlled CS)
    pub fn end_transfer(&mut self) {
        self.cs_mode_word()
    }

    /// Releases the SPI peripheral and associated pins
    pub fn free(self) -> (SPI, PINS) {
        (self.spi, self.pins)
    }
}

impl<SPI: SpiX, PINS> embedded_hal::spi::FullDuplex<u8> for Spi<SPI, PINS> {
    type Error = Infallible;

    fn read(&mut self) -> nb::Result<u8, Infallible> {
        let rxdata = self.spi.rxdata.read();

        if rxdata.empty().bit_is_set() {
            Err(nb::Error::WouldBlock)
        } else {
            Ok(rxdata.data().bits())
        }
    }

    fn send(&mut self, byte: u8) -> nb::Result<(), Infallible> {
        let txdata = self.spi.txdata.read();

        if txdata.full().bit_is_set() {
            Err(nb::Error::WouldBlock)
        } else {
            self.spi.txdata.write(|w| unsafe { w.data().bits(byte) });
            Ok(())
        }
    }
}

impl<SPI: SpiX, PINS> embedded_hal::blocking::spi::Transfer<u8> for Spi<SPI, PINS> {
    type Error = Infallible;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w[u8], Self::Error> {
        // Ensure that RX FIFO is empty
        while self.spi.rxdata.read().empty().bit_is_clear() { }

        self.cs_mode_frame();

        let mut iwrite = 0;
        let mut iread = 0;
        while iwrite < words.len() || iread < words.len() {
            if iwrite < words.len() && self.spi.txdata.read().full().bit_is_clear() {
                let byte = unsafe { words.get_unchecked(iwrite) };
                iwrite += 1;
                self.spi.txdata.write(|w| unsafe { w.data().bits(*byte) });
            }

            if iread < iwrite {
                let data = self.spi.rxdata.read();
                if data.empty().bit_is_clear() {
                    unsafe { *words.get_unchecked_mut(iread) = data.data().bits() };
                    iread += 1;
                }
            }
        }

        self.cs_mode_word();

        Ok(words)
    }
}

impl<SPI: SpiX, PINS> embedded_hal::blocking::spi::Write<u8> for Spi<SPI, PINS> {
    type Error = Infallible;

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        // Ensure that RX FIFO is empty
        while self.spi.rxdata.read().empty().bit_is_clear() { }

        self.cs_mode_frame();

        let mut iwrite = 0;
        let mut iread = 0;
        while iwrite < words.len() || iread < words.len() {
            if iwrite < words.len() && self.spi.txdata.read().full().bit_is_clear() {
                let byte = unsafe { words.get_unchecked(iwrite) };
                iwrite += 1;
                self.spi.txdata.write(|w| unsafe { w.data().bits(*byte) });
            }

            if iread < iwrite {
                // Read and discard byte, if any
                if self.spi.rxdata.read().empty().bit_is_clear() {
                    iread += 1;
                }
            }
        }

        self.cs_mode_word();

        Ok(())
    }
}

impl<SPI: SpiX, PINS> embedded_hal::blocking::spi::WriteIter<u8> for Spi<SPI, PINS> {
    type Error = Infallible;

    fn write_iter<WI>(&mut self, words: WI) -> Result<(), Self::Error>
    where
        WI: IntoIterator<Item=u8>
    {
        let mut iter = words.into_iter();

        // Ensure that RX FIFO is empty
        while self.spi.rxdata.read().empty().bit_is_clear() { }

        self.cs_mode_frame();

        let mut read_count = 0;
        let mut has_data = true;
        while has_data || read_count > 0 {
            if has_data && self.spi.txdata.read().full().bit_is_clear() {
                if let Some(byte) = iter.next() {
                    self.spi.txdata.write(|w| unsafe { w.data().bits(byte) });
                    read_count += 1;
                } else {
                    has_data = false;
                }
            }

            if read_count > 0 {
                // Read and discard byte, if any
                if self.spi.rxdata.read().empty().bit_is_clear() {
                    read_count -= 1;
                }
            }
        }

        self.cs_mode_word();

        Ok(())
    }
}

#[cfg(feature = "async-traits")]
mod async_impls {
    use core::convert::Infallible;
    use core::future::Future;
    use core::pin::Pin;
    use core::task::{Context, Poll};
    use async_embedded_traits::spi::{AsyncTransfer, AsyncWrite, AsyncWriteIter};
    use super::{Spi, SpiX, qspi0::RegisterBlock};

    impl<SPI: SpiX, PINS> AsyncTransfer for Spi<SPI, PINS> {
        type Error = Infallible;
        type TransferFuture<'t> = AsyncTransferFuture<'t>;

        fn async_transfer<'a>(&'a mut self, data: &'a mut [u8]) -> Self::TransferFuture<'a> {
            // Ensure that RX FIFO is empty
            while self.spi.rxdata.read().empty().bit_is_clear() { }

            self.cs_mode_frame();

            AsyncTransferFuture {
                spi: &self.spi,
                data,
                iwrite: 0,
                iread: 0,
            }
        }
    }

    pub struct AsyncTransferFuture<'a> {
        spi: &'a RegisterBlock,
        data: &'a mut [u8],
        iwrite: usize,
        iread: usize,
    }

    impl<'a> Future for AsyncTransferFuture<'a> {
        type Output = Result<(), Infallible>;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let data_len = self.data.len();

            while self.iwrite < data_len || self.iread < data_len {
                if self.iwrite < data_len && self.spi.txdata.read().full().bit_is_clear() {
                    let iwrite = self.iwrite;
                    let byte = unsafe { *self.data.get_unchecked(iwrite) };
                    self.iwrite = iwrite + 1;
                    self.spi.txdata.write(|w| unsafe { w.data().bits(byte) });
                }

                if self.iread < self.iwrite {
                    let data = self.spi.rxdata.read();
                    if data.empty().bit_is_clear() {
                        let iread = self.iread;
                        unsafe { *self.data.get_unchecked_mut(iread) = data.data().bits() };
                        self.iread = iread + 1;
                    } else {
                        cx.waker().wake_by_ref();
                        return Poll::Pending;
                    }
                }
            }

            // self.cs_mode_word();
            if self.spi.csmode.read().bits() != 3 {
                self.spi.csmode.write(|w| unsafe { w.bits(0) });
            }

            Poll::Ready(Ok(()))
        }
    }

    impl<SPI: SpiX, PINS> AsyncWrite for Spi<SPI, PINS> {
        type Error = Infallible;
        type WriteFuture<'t> = AsyncWriteFuture<'t>;

        #[inline(never)]
        fn async_write<'a>(&'a mut self, data: &'a [u8]) -> Self::WriteFuture<'a> {
            // Ensure that RX FIFO is empty
            while self.spi.rxdata.read().empty().bit_is_clear() { }

            self.cs_mode_frame();

            AsyncWriteFuture {
                spi: &self.spi,
                data,
                iwrite: 0,
                iread: 0,
            }
        }
    }

    pub struct AsyncWriteFuture<'a> {
        spi: &'a RegisterBlock,
        data: &'a [u8],
        iwrite: usize,
        iread: usize,
    }

    impl<'a> Future for AsyncWriteFuture<'a> {
        type Output = Result<(), Infallible>;

        //#[inline(never)]
        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let data_len = self.data.len();

            while self.iwrite < data_len || self.iread < data_len {
                if self.iwrite < data_len && self.spi.txdata.read().full().bit_is_clear() {
                    let iwrite = self.iwrite;
                    let byte = unsafe { *self.data.get_unchecked(iwrite) };
                    self.iwrite = iwrite + 1;
                    self.spi.txdata.write(|w| unsafe { w.data().bits(byte) });
                }

                if self.iread < self.iwrite {
                    // Read and discard byte, if any
                    if self.spi.rxdata.read().empty().bit_is_clear() {
                        self.iread += 1;
                    } else {
                        cx.waker().wake_by_ref();
                        return Poll::Pending;
                    }
                }
            }

            // self.cs_mode_word();
            if self.spi.csmode.read().bits() != 3 {
                self.spi.csmode.write(|w| unsafe { w.bits(0) });
            }

            Poll::Ready(Ok(()))
        }
    }

    impl<SPI: SpiX, PINS> AsyncWriteIter for Spi<SPI, PINS> {
        type Error = Infallible;
        type WriteIterFuture<'t> = AsyncWriteIterFuture<'t>;

        #[inline(never)]
        fn async_write_iter<'a>(&'a mut self, iter: &'a mut dyn Iterator<Item=u8>) -> Self::WriteIterFuture<'a> {
            // Ensure that RX FIFO is empty
            while self.spi.rxdata.read().empty().bit_is_clear() { }

            self.cs_mode_frame();

            AsyncWriteIterFuture {
                spi: &self.spi,
                iter,
                has_data: true,
                read_count: 0,
            }
        }
    }

    pub struct AsyncWriteIterFuture<'a> {
        spi: &'a RegisterBlock,
        iter: &'a mut dyn Iterator<Item=u8>,
        has_data: bool,
        read_count: u8,
    }

    impl<'a> Future for AsyncWriteIterFuture<'a> {
        type Output = Result<(), Infallible>;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            while self.has_data || self.read_count > 0 {
                if self.has_data && self.spi.txdata.read().full().bit_is_clear() {
                    if let Some(byte) = self.iter.next() {
                        self.spi.txdata.write(|w| unsafe { w.data().bits(byte) });
                        self.read_count += 1;
                    } else {
                        self.has_data = false;
                    }
                }

                if self.read_count > 0 {
                    // Read and discard byte, if any
                    if self.spi.rxdata.read().empty().bit_is_clear() {
                        self.read_count -= 1;
                    } else {
                        cx.waker().wake_by_ref();
                        return Poll::Pending;
                    }
                }
            }

            // self.cs_mode_word();
            if self.spi.csmode.read().bits() != 3 {
                self.spi.csmode.write(|w| unsafe { w.bits(0) });
            }

            Poll::Ready(Ok(()))
        }
    }
}

// Backward compatibility
impl<PINS> Spi<QSPI0, PINS> {
    /// Configures the SPI peripheral to operate in full duplex master mode
    #[deprecated(note = "Please use Spi::new function instead")]
    pub fn spi0(spi: QSPI0, pins: PINS, mode: Mode, freq: Hertz, clocks: Clocks) -> Self
    where
        PINS: Pins<QSPI0>
    {
        Self::new(spi, pins, mode, freq, clocks)
    }
}

impl<PINS> Spi<QSPI1, PINS> {
    /// Configures the SPI peripheral to operate in full duplex master mode
    #[deprecated(note = "Please use Spi::new function instead")]
    pub fn spi1(spi: QSPI1, pins: PINS, mode: Mode, freq: Hertz, clocks: Clocks) -> Self
        where
            PINS: Pins<QSPI1>
    {
        Self::new(spi, pins, mode, freq, clocks)
    }
}

impl<PINS> Spi<QSPI2, PINS> {
    /// Configures the SPI peripheral to operate in full duplex master mode
    #[deprecated(note = "Please use Spi::new function instead")]
    pub fn spi2(spi: QSPI2, pins: PINS, mode: Mode, freq: Hertz, clocks: Clocks) -> Self
        where
            PINS: Pins<QSPI2>
    {
        Self::new(spi, pins, mode, freq, clocks)
    }
}
