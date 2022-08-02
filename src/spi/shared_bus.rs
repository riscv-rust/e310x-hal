use core::cell::RefCell;
use core::ops::Deref;
use embedded_hal::spi::{ErrorKind, ErrorType};
pub use embedded_hal::spi::{Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3};
use riscv::interrupt;
use riscv::interrupt::Mutex;

use super::{PinCS, PinsNoCS, SpiBus, SpiConfig, SpiSharedDevice, SpiX};

/// Newtype for RefCell<Spi> locked behind a Mutex.
/// Used to hold the [SpiBus] instance so it can be used for multiple [SpiSharedDevice] instances.
pub struct SharedBus<SPI, PINS>(Mutex<RefCell<SpiBus<SPI, PINS>>>);

impl<SPI, PINS> ErrorType for SharedBus<SPI, PINS> {
    type Error = ErrorKind;
}

impl<SPI, PINS> SharedBus<SPI, PINS>
where
    SPI: SpiX,
    PINS: PinsNoCS<SPI>,
{
    pub(crate) fn new(bus: SpiBus<SPI, PINS>) -> Self {
        Self(Mutex::new(RefCell::new(bus)))
    }

    /// Create a new shared device on this SPI bus.
    pub fn new_device<'bus, CS>(
        &'bus self,
        cs: CS,
        config: &SpiConfig,
    ) -> SpiSharedDevice<'bus, SPI, PINS, CS>
    where
        CS: PinCS<SPI>,
    {
        SpiSharedDevice::new(self, cs, config)
    }
}

impl<SPI, PINS> SharedBus<SPI, PINS>
where
    SPI: SpiX,
    PINS: PinsNoCS<SPI>,
{
    /// Set HOLD CS mode to per-frame operation, unless CSMODE is set to OFF
    pub fn start_frame(&mut self) {
        interrupt::free(|cs| {
            let mut bus = self.0.borrow(*cs).borrow_mut();
            bus.start_frame();
        });
    }

    /// Finishes transfer by deasserting CS (only for hardware-controlled CS)
    pub fn end_frame(&mut self) {
        interrupt::free(|cs| {
            let mut bus = self.0.borrow(*cs).borrow_mut();
            bus.end_frame();
        });
    }

    /// Releases the SPI peripheral and associated pins
    pub fn release(self) -> (SPI, PINS) {
        let bus = self.0.into_inner().into_inner();

        (bus.spi, bus.pins)
    }
}

impl<SPI, PINS> Deref for SharedBus<SPI, PINS> {
    type Target = Mutex<RefCell<SpiBus<SPI, PINS>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
