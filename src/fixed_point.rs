#![no_std]

use soroban_sdk::panic_with_error;

#[derive(Clone, Copy, Debug)]
pub struct FixedPoint {
    pub value: i128,
    pub precision: i128,
}

impl FixedPoint {
    pub fn new(value: i128, precision: i128) -> Self {
        if precision <= 0 {
            panic!("precision must be positive");
        }
        FixedPoint { value, precision }
    }

    pub fn from_u128(value: u128, precision: i128) -> Self {
        FixedPoint {
            value: (value as i128) * precision,
            precision,
        }
    }

    pub fn to_u128(&self) -> u128 {
        (self.value / self.precision) as u128
    }

    pub fn add(&self, other: &FixedPoint) -> Self {
        if self.precision != other.precision {
            panic!("precision mismatch");
        }
        FixedPoint {
            value: self.value.checked_add(other.value).unwrap_or_else(|| panic_with_overflow()),
            precision: self.precision,
        }
    }

    pub fn sub(&self, other: &FixedPoint) -> Self {
        if self.precision != other.precision {
            panic!("precision mismatch");
        }
        FixedPoint {
            value: self.value.checked_sub(other.value).unwrap_or_else(|| panic_with_overflow()),
            precision: self.precision,
        }
    }

    pub fn mul(&self, other: &FixedPoint) -> Self {
        let product = self.value.checked_mul(other.value).unwrap_or_else(|| panic_with_overflow());
        FixedPoint {
            value: product / other.precision,
            precision: self.precision.max(other.precision),
        }
    }

    pub fn div(&self, other: &FixedPoint) -> Self {
        if other.value == 0 {
            panic!("division by zero");
        }
        let scaled = self.value.checked_mul(other.precision).unwrap_or_else(|| panic_with_overflow());
        FixedPoint {
            value: scaled / other.value,
            precision: self.precision,
        }
    }

    pub fn sqrt(&self) -> Self {
        if self.value < 0 {
            panic!("sqrt of negative number");
        }
        let x = self.value;
        let precision = self.precision;

        if x == 0 {
            return FixedPoint { value: 0, precision };
        }

        let mut z = (x + precision) / 2;
        let mut prev = 0;

        while (z - prev).abs() > 1 {
            prev = z;
            z = (z + (x * precision) / z) / 2;
        }

        FixedPoint { value: z, precision }
    }

    pub fn square(&self) -> Self {
        self.mul(self)
    }
}

fn panic_with_overflow() -> ! {
    panic!("fixed point overflow");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_point_sqrt() {
        let precision = 1_000_000_000;
        let fp = FixedPoint::new(400_000_000_000, precision);
        let result = fp.sqrt();
        assert_eq!(result.value / precision, 20);
    }

    #[test]
    fn test_fixed_point_mul() {
        let precision = 1_000_000_000;
        let a = FixedPoint::new(5 * precision, precision);
        let b = FixedPoint::new(3 * precision, precision);
        let result = a.mul(&b);
        assert_eq!(result.value / precision, 15);
    }

    #[test]
    fn test_fixed_point_div() {
        let precision = 1_000_000_000;
        let a = FixedPoint::new(10 * precision, precision);
        let b = FixedPoint::new(2 * precision, precision);
        let result = a.div(&b);
        assert_eq!(result.value / precision, 5);
    }
}
