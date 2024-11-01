use crate::prelude::*;



#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bytes(pub usize);

impl std::fmt::Display for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const TWO_KILOBYTES: usize = 1 << 11;
        const TWO_KILOBYTES_MINUS_ONE: usize = TWO_KILOBYTES - 1;
        const TWO_MEGABYTES: usize = 1 << 21;
        const TWO_MEGABYTES_MINUS_ONE: usize = TWO_MEGABYTES - 1;
        const TWO_GIGABYTES: usize = 1 << 31;
        const TWO_GIGABYTES_MINUS_ONE: usize = TWO_GIGABYTES - 1;

        match self.0 {
            0..=TWO_KILOBYTES_MINUS_ONE => write!(f, "{} bytes", self.0),
            TWO_KILOBYTES..=TWO_MEGABYTES_MINUS_ONE => write!(f, "{} kilobytes", self.0 >> 10),
            TWO_MEGABYTES..=TWO_GIGABYTES_MINUS_ONE => write!(f, "{} megabytes", self.0 >> 20),
            _ => write!(f, "{} gigabytes", self.0 >> 30),
        }
    }
}