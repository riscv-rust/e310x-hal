/// Helper traits for SPI pins
use core::ops::Deref;
use e310x::{qspi0, QSPI0, QSPI1, QSPI2};

#[doc(hidden)]
pub trait SpiX: Deref<Target = qspi0::RegisterBlock> + private::Sealed {}
impl SpiX for QSPI0 {}
impl SpiX for QSPI1 {}
impl SpiX for QSPI2 {}

/// SPI pins - DO NOT IMPLEMENT THIS TRAIT
///
/// This trait is implemented for pin tuples (), (MOSI, MISO, SCK) and (MOSI, MISO, SCK, CS)
/// and combinations without MOSI/MISO
pub trait Pins<SPI>: private::Sealed {
    #[doc(hidden)]
    const CS_INDEX: Option<u32>;
}

/// SPI pins without CS - DO NOT IMPLEMENT THIS TRAIT
///
/// This trait is implemented for pin tuples (), (MOSI, MISO, SCK) only without CS pin
/// and combinations without MOSI/MISO
pub trait PinsNoCS<SPI>: Pins<SPI> {}

/// SPI Chip Select pin - DO NOT IMPLEMENT THIS TRAIT
///
/// This trait is implemented for chip select pins only
pub trait PinCS<SPI>: private::Sealed {
    #[doc(hidden)]
    const CS_INDEX: u32;
}

/* SPI0 pins */
impl Pins<QSPI0> for () {
    const CS_INDEX: Option<u32> = Some(0);
}

/* SPI1 pins */
mod spi1_impl {
    use super::{PinCS, Pins, PinsNoCS, QSPI1};
    use crate::gpio::gpio0;
    use crate::gpio::{NoInvert, IOF0};

    type Mosi = gpio0::Pin3<IOF0<NoInvert>>;
    type Miso = gpio0::Pin4<IOF0<NoInvert>>;
    type Sck = gpio0::Pin5<IOF0<NoInvert>>;
    type Cs0 = gpio0::Pin2<IOF0<NoInvert>>;
    type Cs1 = gpio0::Pin8<IOF0<NoInvert>>;
    type Cs2 = gpio0::Pin9<IOF0<NoInvert>>;
    type Cs3 = gpio0::Pin10<IOF0<NoInvert>>;

    // ensure only the correct CS pins can be used to make SpiSharedDevice instances
    impl PinCS<QSPI1> for Cs0 {
        const CS_INDEX: u32 = 0;
    }
    impl PinCS<QSPI1> for Cs1 {
        const CS_INDEX: u32 = 1;
    }
    impl PinCS<QSPI1> for Cs2 {
        const CS_INDEX: u32 = 2;
    }
    impl PinCS<QSPI1> for Cs3 {
        const CS_INDEX: u32 = 3;
    }

    impl PinsNoCS<QSPI1> for (Mosi, Miso, Sck) {}
    impl PinsNoCS<QSPI1> for (Mosi, (), Sck) {}
    impl PinsNoCS<QSPI1> for ((), Miso, Sck) {}

    impl Pins<QSPI1> for (Mosi, Miso, Sck) {
        const CS_INDEX: Option<u32> = None;
    }
    impl Pins<QSPI1> for (Mosi, (), Sck) {
        const CS_INDEX: Option<u32> = None;
    }
    impl Pins<QSPI1> for ((), Miso, Sck) {
        const CS_INDEX: Option<u32> = None;
    }
    impl Pins<QSPI1> for (Mosi, Miso, Sck, Cs0) {
        const CS_INDEX: Option<u32> = Some(0);
    }
    impl Pins<QSPI1> for (Mosi, (), Sck, Cs0) {
        const CS_INDEX: Option<u32> = Some(0);
    }
    impl Pins<QSPI1> for ((), Miso, Sck, Cs0) {
        const CS_INDEX: Option<u32> = Some(0);
    }
    impl Pins<QSPI1> for (Mosi, Miso, Sck, Cs1) {
        const CS_INDEX: Option<u32> = Some(1);
    }
    impl Pins<QSPI1> for (Mosi, (), Sck, Cs1) {
        const CS_INDEX: Option<u32> = Some(1);
    }
    impl Pins<QSPI1> for ((), Miso, Sck, Cs1) {
        const CS_INDEX: Option<u32> = Some(1);
    }
    impl Pins<QSPI1> for (Mosi, Miso, Sck, Cs2) {
        const CS_INDEX: Option<u32> = Some(2);
    }
    impl Pins<QSPI1> for (Mosi, (), Sck, Cs2) {
        const CS_INDEX: Option<u32> = Some(2);
    }
    impl Pins<QSPI1> for ((), Miso, Sck, Cs2) {
        const CS_INDEX: Option<u32> = Some(2);
    }
    impl Pins<QSPI1> for (Mosi, Miso, Sck, Cs3) {
        const CS_INDEX: Option<u32> = Some(3);
    }
    impl Pins<QSPI1> for (Mosi, (), Sck, Cs3) {
        const CS_INDEX: Option<u32> = Some(3);
    }
    impl Pins<QSPI1> for ((), Miso, Sck, Cs3) {
        const CS_INDEX: Option<u32> = Some(3);
    }

    // seal the "private" traits
    mod spi1_private {
        use super::super::private::Sealed;
        use super::*;

        impl Sealed for Cs0 {}
        impl Sealed for Cs1 {}
        impl Sealed for Cs2 {}
        impl Sealed for Cs3 {}
        impl Sealed for (Mosi, Miso, Sck) {}
        impl Sealed for (Mosi, (), Sck) {}
        impl Sealed for ((), Miso, Sck) {}
        impl Sealed for (Mosi, Miso, Sck, Cs0) {}
        impl Sealed for (Mosi, (), Sck, Cs0) {}
        impl Sealed for ((), Miso, Sck, Cs0) {}
        impl Sealed for (Mosi, Miso, Sck, Cs1) {}
        impl Sealed for (Mosi, (), Sck, Cs1) {}
        impl Sealed for ((), Miso, Sck, Cs1) {}
        impl Sealed for (Mosi, Miso, Sck, Cs2) {}
        impl Sealed for (Mosi, (), Sck, Cs2) {}
        impl Sealed for ((), Miso, Sck, Cs2) {}
        impl Sealed for (Mosi, Miso, Sck, Cs3) {}
        impl Sealed for (Mosi, (), Sck, Cs3) {}
        impl Sealed for ((), Miso, Sck, Cs3) {}
    }
}

/* SPI2 pins */
mod spi2_impl {
    use super::{PinCS, Pins, PinsNoCS, QSPI2};
    use crate::gpio::gpio0;
    use crate::gpio::{NoInvert, IOF0};

    type Mosi = gpio0::Pin27<IOF0<NoInvert>>;
    type Miso = gpio0::Pin28<IOF0<NoInvert>>;
    type Sck = gpio0::Pin29<IOF0<NoInvert>>;
    type Cs0 = gpio0::Pin26<IOF0<NoInvert>>;

    impl PinCS<QSPI2> for Cs0 {
        const CS_INDEX: u32 = 0;
    }

    impl PinsNoCS<QSPI2> for (Mosi, Miso, Sck) {}
    impl PinsNoCS<QSPI2> for (Mosi, (), Sck) {}
    impl PinsNoCS<QSPI2> for ((), Miso, Sck) {}

    impl Pins<QSPI2> for (Mosi, Miso, Sck) {
        const CS_INDEX: Option<u32> = None;
    }
    impl Pins<QSPI2> for (Mosi, (), Sck) {
        const CS_INDEX: Option<u32> = None;
    }
    impl Pins<QSPI2> for ((), Miso, Sck) {
        const CS_INDEX: Option<u32> = None;
    }
    impl Pins<QSPI2> for (Mosi, Miso, Sck, Cs0) {
        const CS_INDEX: Option<u32> = Some(0);
    }
    impl Pins<QSPI2> for (Mosi, (), Sck, Cs0) {
        const CS_INDEX: Option<u32> = Some(0);
    }
    impl Pins<QSPI2> for ((), Miso, Sck, Cs0) {
        const CS_INDEX: Option<u32> = Some(0);
    }

    // seal the "private" traits
    mod spi2_private {
        use super::super::private::Sealed;
        use super::*;

        impl Sealed for Cs0 {}
        impl Sealed for (Mosi, Miso, Sck) {}
        impl Sealed for (Mosi, (), Sck) {}
        impl Sealed for ((), Miso, Sck) {}
        impl Sealed for (Mosi, Miso, Sck, Cs0) {}
        impl Sealed for (Mosi, (), Sck, Cs0) {}
        impl Sealed for ((), Miso, Sck, Cs0) {}
    }
}

// seal the "private" traits
mod private {
    pub trait Sealed {}

    impl Sealed for () {}

    impl Sealed for super::QSPI0 {}
    impl Sealed for super::QSPI1 {}
    impl Sealed for super::QSPI2 {}
}
