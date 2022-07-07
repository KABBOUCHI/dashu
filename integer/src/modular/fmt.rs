//! Formatting modular rings and modular numbers.

use crate::modular::{
    modulo::{Modulo, ModuloLarge, ModuloRepr, ModuloSingle},
    modulo_ring::{ModuloRing, ModuloRingLarge, ModuloRingRepr, ModuloRingSingle},
};
use core::fmt::{self, Binary, Debug, Display, Formatter, LowerHex, Octal, UpperHex};

macro_rules! impl_fmt {
    ($t:ident) => {
        impl $t for ModuloRing {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                match self.repr() {
                    ModuloRingRepr::Single(self_small) => $t::fmt(self_small, f),
                    ModuloRingRepr::Large(self_large) => $t::fmt(self_large, f),
                }
            }
        }

        impl $t for ModuloRingSingle {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                f.write_str("mod ")?;
                $t::fmt(&self.modulus(), f)
            }
        }

        impl $t for ModuloRingLarge {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                f.write_str("mod ")?;
                $t::fmt(&self.modulus(), f)
            }
        }

        impl $t for Modulo<'_> {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                match self.repr() {
                    ModuloRepr::Small(self_small) => $t::fmt(self_small, f),
                    ModuloRepr::Large(self_large) => $t::fmt(self_large, f),
                }
            }
        }

        impl $t for ModuloSingle<'_> {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                $t::fmt(&self.raw().residue(self.ring()), f)?;
                f.write_str(" (")?;
                $t::fmt(self.ring(), f)?;
                f.write_str(")")
            }
        }

        impl $t for ModuloLarge<'_> {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                $t::fmt(&self.residue(), f)?;
                f.write_str(" (")?;
                $t::fmt(self.ring(), f)?;
                f.write_str(")")
            }
        }
    };
}

impl_fmt!(Display);
impl_fmt!(Debug);
impl_fmt!(Binary);
impl_fmt!(Octal);
impl_fmt!(LowerHex);
impl_fmt!(UpperHex);
