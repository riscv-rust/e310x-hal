use core::convert::Infallible;

use embedded_hal::{
    blocking::spi::{Transfer, Write, WriteIter},
    spi::FullDuplex,
};
use riscv::interrupt;

use super::{PinCS, Pins, PinsNoCS, SharedBus, SpiConfig, SpiX};

/// SPI shared device abstraction
pub struct SpiSharedDevice<'bus, SPI, PINS, CS> {
    bus: &'bus SharedBus<SPI, PINS>,
    cs: CS,
    config: SpiConfig,
}

impl<'bus, SPI, PINS, CS> SpiSharedDevice<'bus, SPI, PINS, CS>
where
    SPI: SpiX,
    PINS: PinsNoCS<SPI>,
    CS: PinCS<SPI>,
{
    /// Create shared [SpiSharedDevice] using the existing [SharedBus]
    /// and given [SpiConfig]. The config gets cloned.
    pub fn new(bus: &'bus SharedBus<SPI, PINS>, cs: CS, config: &SpiConfig) -> Self
    where
        PINS: PinsNoCS<SPI>,
    {
        Self {
            bus,
            cs,
            config: config.clone(),
        }
    }

    /// Releases the CS pin back
    pub fn release(self) -> CS {
        self.cs
    }
}

impl<SPI, PINS, CS> FullDuplex<u8> for SpiSharedDevice<'_, SPI, PINS, CS>
where
    SPI: SpiX,
    PINS: Pins<SPI>,
    CS: PinCS<SPI>,
{
    type Error = Infallible;

    fn try_read(&mut self) -> nb::Result<u8, Infallible> {
        interrupt::free(|cs| {
            let mut bus = self.bus.borrow(*cs).borrow_mut();

            bus.configure(&self.config, Some(CS::CS_INDEX));

            bus.read()
        })
    }

    fn try_send(&mut self, byte: u8) -> nb::Result<(), Infallible> {
        interrupt::free(|cs| {
            let mut bus = self.bus.borrow(*cs).borrow_mut();

            bus.configure(&self.config, Some(CS::CS_INDEX));

            bus.send(byte)
        })
    }
}

impl<SPI, PINS, CS> Transfer<u8> for SpiSharedDevice<'_, SPI, PINS, CS>
where
    SPI: SpiX,
    PINS: Pins<SPI>,
    CS: PinCS<SPI>,
{
    type Error = Infallible;

    fn try_transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        interrupt::free(move |cs| {
            let mut bus = self.bus.borrow(*cs).borrow_mut();

            bus.configure(&self.config, Some(CS::CS_INDEX));

            bus.start_frame();
            let result = bus.transfer(words);
            bus.end_frame();

            result
        })
    }
}

impl<SPI, PINS, CS> Write<u8> for SpiSharedDevice<'_, SPI, PINS, CS>
where
    SPI: SpiX,
    PINS: Pins<SPI>,
    CS: PinCS<SPI>,
{
    type Error = Infallible;

    fn try_write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        interrupt::free(|cs| {
            let mut bus = self.bus.borrow(*cs).borrow_mut();

            bus.configure(&self.config, Some(CS::CS_INDEX));

            bus.start_frame();
            let result = bus.write(words);
            bus.end_frame();

            result
        })
    }
}

impl<SPI, PINS, CS> WriteIter<u8> for SpiSharedDevice<'_, SPI, PINS, CS>
where
    SPI: SpiX,
    PINS: Pins<SPI>,
    CS: PinCS<SPI>,
{
    type Error = Infallible;

    fn try_write_iter<WI>(&mut self, words: WI) -> Result<(), Self::Error>
    where
        WI: IntoIterator<Item = u8>,
    {
        interrupt::free(|cs| {
            let mut bus = self.bus.borrow(*cs).borrow_mut();

            bus.configure(&self.config, Some(CS::CS_INDEX));

            bus.start_frame();
            let result = bus.write_iter(words);
            bus.end_frame();

            result
        })
    }
}
