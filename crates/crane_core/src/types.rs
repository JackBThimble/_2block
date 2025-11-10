//! Physical quantity types using uom (Units of Measurement)
//! All types use f64 for precision in crane calculations

use uom::si::f64::*;
pub use uom::si::f64::{
    Acceleration, Angle, AngularVelocity, Force, Length, Mass, Pressure, Time, Velocity,
};

pub mod length {
    pub use uom::si::length::{centimeter, foot, inch, kilometer, meter, millimeter};
}

pub mod mass {
    pub use uom::si::mass::{gram, kilogram, megagram as metric_ton, pound};
}

pub mod force {
    pub use uom::si::force::{kilonewton, newton, pound_force};
}

pub mod angle {
    pub use uom::si::angle::{degree, radian};
}

pub mod time {
    pub use uom::si::time::{hour, minute, second};
}

pub mod angular_velocity {
    pub use uom::si::angular_velocity::{degree_per_second, radian_per_second};
}

pub mod velocity {
    pub use uom::si::velocity::{
        foot_per_minute, foot_per_second, kilometer_per_hour, meter_per_second, mile_per_hour,
    };
}

pub mod pressure {
    pub use uom::si::pressure::{kilopascal, pascal, psi};
}
