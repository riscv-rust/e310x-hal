use embedded_hal::spi::blocking::{SpiBus as SpiBusTransfer, SpiBusFlush};
use embedded_hal::spi::blocking::{SpiBusRead, SpiBusWrite};
use embedded_hal::spi::ErrorType;
pub use embedded_hal::spi::{ErrorKind, Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3};

use super::{Pins, PinsNoCS, SharedBus, SpiConfig, SpiExclusiveDevice, SpiX};

/// SPI bus abstraction
pub struct SpiBus<SPI, PINS> {
    pub(crate) spi: SPI,
    pub(crate) pins: PINS,
}

impl<SPI, PINS> SpiBus<SPI, PINS>
where
    SPI: SpiX,
{
    /// Construct the [SpiBus] for use with [SpiSharedDevice](super::SpiSharedDevice) or [SpiExclusiveDevice]
    pub fn new(spi: SPI, pins: PINS) -> Self
    where
        PINS: Pins<SPI>,
    {
        Self { spi, pins }
    }

    /// Releases the SPI peripheral and associated pins
    pub fn release(self) -> (SPI, PINS) {
        (self.spi, self.pins)
    }

    /// Configure the [SpiBus] with given [SpiConfig]
    pub(crate) fn configure(&mut self, config: &SpiConfig, cs_index: Option<u32>)
    where
        PINS: Pins<SPI>,
    {
        self.spi
            .sckdiv
            .write(|w| unsafe { w.div().bits(config.clock_divisor as u16) });

        if let Some(index) = cs_index {
            self.spi.csid.write(|w| unsafe { w.bits(index) });
        }
        self.spi.csmode.write(|w| w.mode().variant(config.cs_mode));

        // Set CS pin polarity to high
        self.spi.csdef.reset();

        // Set SPI mode
        let phase = config.mode.phase == Phase::CaptureOnSecondTransition;
        let polarity = config.mode.polarity == Polarity::IdleHigh;
        self.spi
            .sckmode
            .write(|w| w.pha().bit(phase).pol().bit(polarity));

        self.spi.fmt.write(|w| unsafe {
            w.proto().single();
            w.endian().big(); // Transmit most-significant bit (MSB) first
            w.dir().rx();
            w.len().bits(8)
        });

        // Set watermark levels
        self.spi
            .txmark
            .write(|w| unsafe { w.txmark().bits(config.txmark) });
        self.spi
            .rxmark
            .write(|w| unsafe { w.rxmark().bits(config.rxmark) });

        // set delays
        self.spi.delay0.write(|w| unsafe {
            w.cssck().bits(config.delays.cssck); // delay between assert and clock
            w.sckcs().bits(config.delays.sckcs) // delay between clock and de-assert
        });
        self.spi.delay1.write(|w| unsafe {
            w.intercs().bits(config.delays.intercs); // delay between CS re-assets
            w.interxfr().bits(config.delays.interxfr) // intra-frame delay without CS re-asserts
        });

        self.end_frame(); // ensure CS is de-asserted before we begin
    }

    fn wait_for_rxfifo(&self) {
        // Ensure that RX FIFO is empty
        while self.spi.rxdata.read().empty().bit_is_clear() {}
    }

    /// Starts frame by flagging CS assert, unless CSMODE = OFF
    pub(crate) fn start_frame(&mut self) {
        if !self.spi.csmode.read().mode().is_off() {
            self.spi.csmode.write(|w| w.mode().hold());
        }
    }

    /// Finishes frame flagging CS deassert, unless CSMODE = OFF
    pub(crate) fn end_frame(&mut self) {
        if !self.spi.csmode.read().mode().is_off() {
            self.spi.csmode.write(|w| w.mode().auto());
        }
    }

    /// Transfer implementation out of trait for reuse in Read and Write
    fn perform_transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), ErrorKind> {
        let mut iwrite = 0;
        let mut iread = 0;
        let bytes = core::cmp::max(read.len(), write.len());

        // Ensure that RX FIFO is empty
        self.wait_for_rxfifo();

        // go through entire write buffer and read back (even if read buffer is empty)
        // while iwrite < write.len() || iread < write.len() {
        while iwrite < bytes || iread < bytes {
            if iwrite < write.len() && self.spi.txdata.read().full().bit_is_clear() {
                let byte = write.get(iwrite).unwrap_or(&0);
                iwrite += 1;
                self.spi.txdata.write(|w| unsafe { w.data().bits(*byte) });
            }

            if iread < iwrite {
                let data = self.spi.rxdata.read();
                if data.empty().bit_is_clear() {
                    if let Some(d) = read.get_mut(iread) {
                        *d = data.data().bits()
                    };
                    iread += 1;
                }
            }
        }

        Ok(())
    }
}

impl<SPI, PINS> ErrorType for SpiBus<SPI, PINS> {
    type Error = ErrorKind;
}

impl<SPI, PINS> SpiBusFlush for SpiBus<SPI, PINS>
where
    SPI: SpiX,
{
    fn flush(&mut self) -> Result<(), Self::Error> {
        // unnecessary

        Ok(())
    }
}
impl<SPI, PINS> SpiBusRead for SpiBus<SPI, PINS>
where
    SPI: SpiX,
{
    fn read(&mut self, words: &mut [u8]) -> Result<(), ErrorKind> {
        self.perform_transfer(words, &[])
    }
}

impl<SPI, PINS> SpiBusWrite for SpiBus<SPI, PINS>
where
    SPI: SpiX,
{
    fn write(&mut self, words: &[u8]) -> Result<(), ErrorKind> {
        self.perform_transfer(&mut [], words)
    }
}

impl<SPI, PINS> SpiBusTransfer for SpiBus<SPI, PINS>
where
    SPI: SpiX,
{
    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), ErrorKind> {
        self.perform_transfer(read, write)
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), ErrorKind> {
        let mut iwrite = 0;
        let mut iread = 0;

        // Ensure that RX FIFO is empty
        self.wait_for_rxfifo();

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

        Ok(())
    }
}

impl<SPI, PINS> SpiBus<SPI, PINS>
where
    SPI: SpiX,
    PINS: Pins<SPI>,
{
    /// Create a new [SpiExclusiveDevice] for exclusive use on this bus
    pub fn new_device(self, config: &SpiConfig) -> SpiExclusiveDevice<SPI, PINS> {
        SpiExclusiveDevice::new(self, config)
    }
}

impl<SPI, PINS> SpiBus<SPI, PINS>
where
    SPI: SpiX,
    PINS: PinsNoCS<SPI>,
{
    /// Create a [SharedBus] for use with multiple devices.
    pub fn shared(spi: SPI, pins: PINS) -> SharedBus<SPI, PINS> {
        SharedBus::new(Self::new(spi, pins))
    }
}
