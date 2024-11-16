use rust_decimal::Decimal;
use std::sync::atomic::{AtomicI64, Ordering};

// 使用自定义原子Decimal包装
#[derive(Debug)]
pub struct AtomicDecimal {
    // 使用原始数值 * 10^scale 存储
    value: AtomicI64,
    scale: u32,
    order: Ordering,
}

impl Default for AtomicDecimal {
    fn default() -> Self {
        AtomicDecimal::new()
    }
}

impl AtomicDecimal {
    pub fn new() -> Self {
        let scale = 8; // 使用8位精度

        Self {
            value: AtomicI64::new(0),
            scale,
            order: Ordering::SeqCst,
        }
    }

    pub fn load(&self) -> Decimal {
        let raw_value = self.value.load(self.order);
        self.raw_value_to_decimal(raw_value)
    }

    pub fn store(&self, value: Decimal) {
        let raw_value = self.decimal_to_raw_value(value);
        self.value.store(raw_value, self.order);
    }

    pub fn fetch_add(&self, value: Decimal) -> Decimal {
        let raw_value = self.decimal_to_raw_value(value);
        let old_raw = self.value.fetch_add(raw_value, self.order);
        self.raw_value_to_decimal(old_raw)
    }

    pub fn fetch_sub(&self, value: Decimal) -> Decimal {
        let raw_value = self.decimal_to_raw_value(value);
        let old_raw = self.value.fetch_sub(raw_value, self.order);
        self.raw_value_to_decimal(old_raw)
    }

    fn decimal_to_raw_value(&self, value: Decimal) -> i64 {
        (value * Decimal::from(10_i64.pow(self.scale)))
            .floor()
            .to_string()
            .parse::<i64>()
            .unwrap_or_default()
    }

    fn raw_value_to_decimal(&self, raw_value: i64) -> Decimal {
        Decimal::from(raw_value) / Decimal::from(10_i64.pow(self.scale))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_atomic_decimal_basic_ops() {
        let atomic_dec = AtomicDecimal::new();

        // 测试load/store
        atomic_dec.store(dec!(3.14159));
        assert_eq!(atomic_dec.load(), dec!(3.14159));

        atomic_dec.store(dec!(2.71828));
        assert_eq!(atomic_dec.load(), dec!(2.71828));

        // 测试fetch_add
        let old = atomic_dec.fetch_add(dec!(1.0));
        assert_eq!(old, dec!(2.71828));
        assert_eq!(atomic_dec.load(), dec!(3.71828));

        // 测试fetch_sub
        let old = atomic_dec.fetch_sub(dec!(1.21828));
        assert_eq!(old, dec!(3.71828));
        assert_eq!(atomic_dec.load(), dec!(2.5));
    }
}
