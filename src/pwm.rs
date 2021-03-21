//! # Pulse Width Modulation Interface
//! You can use the `Pwm` and `PwmPin` interfaces with this module
//! There are 3 PWM control groups, each one with 3 pins, one per channel
//! 
//! *Warning:* `Pwm0` a max period of 256, as it only has an 8 bit comparison
//! register, the rest of them have a max value of 2^16 as they have 16 bit
//! registers
//!
//! The period counter is incremented once every clock tick, so a clock of 320Mhz
//! means that it's incremented 320 million times per second

use e310x::{pwm0::RegisterBlock, PWM0, PWM1, PWM2};
use embedded_hal::{Pwm, PwmPin};

use crate::gpio::{
    gpio0::{Pin1, Pin11, Pin12, Pin13,Pin19, Pin2, Pin21, Pin22, Pin3},
    NoInvert, IOF1,
};

/// What channel to enable/update the duty for 
#[derive(Clone)]
pub enum Channel {
    /// Channel 1
    Cmp1,
    /// Channel 2
    Cmp2,
    /// Channel 3
    Cmp3,
}
/// Pwm Abstraction
pub trait PwmPeripheral {
    /// Get the corresponding register block
    fn peripheral() -> &'static RegisterBlock;
}

macro_rules! pwm_group {
    ($PWM_PERIPH:ident,$pwm_periph:ident,[
        $($PWMPIN:ident: ($PXi:ident, $CMP:expr),)+
    ]) => {
        /// A PWM control group
        ///
        /// See `embedded-hal::Pwm` for the API
        pub struct $pwm_periph{
        }
        impl PwmPeripheral for $pwm_periph {
            fn peripheral() ->&'static RegisterBlock {
                unsafe { &*$PWM_PERIPH::ptr() }
            }
        }

        impl Pwm for $pwm_periph
        {
            type Duty = u16;

            type Channel = Channel;

            type Time = u16;

            fn disable(&mut self, channel: Self::Channel) {
                match channel {
                    Channel::Cmp1 => Self::peripheral().cmp1.reset(),
                    Channel::Cmp2 => Self::peripheral().cmp2.reset(),
                    Channel::Cmp3 => Self::peripheral().cmp3.reset(),
                };
            }
            /// Enable the zerocomp bit so the counter resets every time it's equal
            /// to the value in Comparator 0
            fn enable(&mut self, channel: Self::Channel) {
                Self::peripheral().cfg.write(|w| w.zerocmp().set_bit());
                Self::peripheral().cfg.write(|w| w.enalways().set_bit());
                match channel {
                    Channel::Cmp1 => Self::peripheral().cmp1.write(|w| unsafe { w.bits(1) }),
                    Channel::Cmp2 => Self::peripheral().cmp2.write(|w| unsafe { w.bits(1) }),
                    Channel::Cmp3 => Self::peripheral().cmp3.write(|w| unsafe { w.bits(1) }),
                }
            }

            fn get_period(&self) -> Self::Time {
                Self::peripheral().cmp0.read().bits() as u16
            }

            fn get_duty(&self, channel: Self::Channel) -> Self::Duty {
                let duty = match channel {
                    Channel::Cmp1 => Self::peripheral().cmp1.read().bits(),
                    Channel::Cmp2 => Self::peripheral().cmp2.read().bits(),
                    Channel::Cmp3 => Self::peripheral().cmp3.read().bits(),
                };
                duty as u16
            }

            fn get_max_duty(&self) -> Self::Duty {
                self.get_period()
            }

            fn set_duty(&mut self, channel: Self::Channel, duty: Self::Duty) {
                let mut duty = duty as u32;
                if duty > self.get_max_duty() as u32{
                    duty = self.get_max_duty() as u32;
                }
                match channel {
                    Channel::Cmp1 => Self::peripheral().cmp1.write(|w| unsafe { w.bits(duty) }),
                    Channel::Cmp2 => Self::peripheral().cmp2.write(|w| unsafe { w.bits(duty) }),
                    Channel::Cmp3 => Self::peripheral().cmp3.write(|w| unsafe { w.bits(duty) }),
                }
            }

            fn set_period<P>(&mut self, period: P)
            where
                P: Into<Self::Time>,
            {
                let period = period.into() as u32;
                Self::peripheral().count.reset();
                Self::peripheral().cmp0.write(|w| unsafe { w.bits(period) });
            }
        }

        $(
        /// A PWM enable pin
        ///
        /// See embedded-hal::PwmPin for the API
        pub struct $PWMPIN {
            pin: $PXi<IOF1<NoInvert>>,
            pwm_group:$pwm_periph,
            channel: Channel,
        }
        impl $PWMPIN {
            /// Create a new PWM pin
            pub fn new<T>(pin: $PXi<T>) -> Self {
                let pin = pin.into_iof1();
                Self {
                    pin,
                    pwm_group:$pwm_periph{},
                    channel: $CMP,
                }
            }
        }

        impl PwmPin for $PWMPIN {
            type Duty = u16;

            fn disable(&mut self) {
                self.pwm_group.disable(self.channel.clone());
            }

            fn enable(&mut self) {
                self.pwm_group.disable(self.channel.clone());
            }

            fn get_duty(&self) -> Self::Duty {
                self.pwm_group.get_duty(self.channel.clone())
            }

            fn get_max_duty(&self) -> Self::Duty {
                self.pwm_group.get_max_duty()
            }

            fn set_duty(&mut self, duty: Self::Duty) {
                self.pwm_group.set_duty(self.channel.clone(), duty)
            }
        }
        )+
    };
}

pwm_group!(
    PWM0,
    Pwm0,
    [
        PwmPin1: (Pin1, Channel::Cmp1),
        PwmPin2: (Pin2, Channel::Cmp2),
        PwmPin3: (Pin3, Channel::Cmp3),
    ]
);
pwm_group!(
    PWM2,
    Pwm2,
    [
        PwmPin11: (Pin11, Channel::Cmp1),
        PwmPin12: (Pin12, Channel::Cmp2),
        PwmPin13: (Pin13, Channel::Cmp3),
    ]
);
pwm_group!(
    PWM1,
    Pwm1,
    [
        PwmPin19: (Pin19, Channel::Cmp1),
        PwmPin21: (Pin21, Channel::Cmp2),
        PwmPin22: (Pin22, Channel::Cmp3),
    ]
);
