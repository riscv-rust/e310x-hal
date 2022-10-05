use embedded_hal::spi::{blocking::SpiDevice, ErrorKind, ErrorType};

use crate::spi::SpiConfig;

use super::{Pins, SpiBus, SpiX};

/// SPI exclusive device abstraction
pub struct SpiExclusiveDevice<SPI, PINS> {
    bus: SpiBus<SPI, PINS>,
}

impl<SPI, PINS> SpiExclusiveDevice<SPI, PINS>
where
    SPI: SpiX,
    PINS: Pins<SPI>,
{
    /// Create [SpiExclusiveDevice] using the existing [SpiBus](super::SpiBus)
    /// with the given [SpiConfig]
    pub fn new(mut bus: SpiBus<SPI, PINS>, config: &SpiConfig) -> Self
    where
        PINS: Pins<SPI>,
    {
        bus.configure(config, PINS::CS_INDEX);

        Self { bus }
    }

    /// Releases the Bus back deconstructing it
    pub fn release(self) -> (SPI, PINS) {
        self.bus.release()
    }
}

impl<SPI, PINS> ErrorType for SpiExclusiveDevice<SPI, PINS> {
    type Error = ErrorKind;
}

impl<SPI, PINS> SpiDevice for SpiExclusiveDevice<SPI, PINS>
where
    SPI: SpiX,
    PINS: Pins<SPI>,
{
    type Bus = SpiBus<SPI, PINS>;

    fn transaction<R>(
        &mut self,
        f: impl FnOnce(&mut Self::Bus) -> Result<R, <Self::Bus as ErrorType>::Error>,
    ) -> Result<R, Self::Error> {
        self.bus.start_frame();
        let result = f(&mut self.bus)?;
        self.bus.end_frame();

        Ok(result)
    }
}
