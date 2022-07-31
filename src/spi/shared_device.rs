use core::ops::DerefMut;

use embedded_hal::spi::{blocking::SpiDevice, ErrorKind, ErrorType};
use riscv::interrupt;

use super::{PinCS, PinsNoCS, SharedBus, SpiBus, SpiConfig, SpiX};

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

impl<SPI, PINS, CS> ErrorType for SpiSharedDevice<'_, SPI, PINS, CS> {
    type Error = ErrorKind;
}

impl<SPI, PINS, CS> SpiDevice for SpiSharedDevice<'_, SPI, PINS, CS>
where
    SPI: SpiX,
    PINS: PinsNoCS<SPI>,
    CS: PinCS<SPI>,
{
    type Bus = SpiBus<SPI, PINS>;
    // type Bus = RefMut<'bus, SpiBus<SPI, PINS>>;

    fn transaction<R>(
        &mut self,
        f: impl FnOnce(&mut Self::Bus) -> Result<R, <Self::Bus as ErrorType>::Error>,
    ) -> Result<R, Self::Error> {
        let mut result = Err(ErrorKind::Other);

        interrupt::free(|cs| {
            let mut bus = self.bus.borrow(*cs).borrow_mut();

            bus.configure(&self.config, Some(CS::CS_INDEX));

            bus.start_frame();
            result = f(bus.deref_mut());
            bus.end_frame();

            0
        });

        result
    }
}
