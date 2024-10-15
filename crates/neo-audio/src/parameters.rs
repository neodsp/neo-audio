use std::{
    ops::RangeInclusive,
    sync::atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicU8, Ordering},
};

pub struct FloatParameter {
    value: AtomicU32,
    name: String,
    default: f32,
    range: RangeInclusive<f32>,
}

impl FloatParameter {
    pub fn new(name: impl Into<String>, default: f32, range: RangeInclusive<f32>) -> Self {
        assert!(range.contains(&default));
        Self {
            value: AtomicU32::new(default.to_bits()),
            name: name.into(),
            default,
            range,
        }
    }

    pub fn value(&self) -> f32 {
        f32::from_bits(self.value.load(Ordering::Relaxed))
    }

    pub fn set_value(&self, val: f32) {
        self.value.store(
            val.clamp(*self.range.start(), *self.range.end()).to_bits(),
            Ordering::Relaxed,
        );
    }

    pub fn range(&self) -> &RangeInclusive<f32> {
        &self.range
    }

    pub fn default(&self) -> f32 {
        self.default
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn atomic(&self) -> &AtomicU32 {
        &self.value
    }
}

pub struct ChoiceParameter {
    value: AtomicU8,
    name: String,
    default: u8,
    choices: Vec<String>,
}

impl ChoiceParameter {
    pub fn new(
        name: impl Into<String>,
        default_index: u8,
        choices: impl Into<Vec<String>>,
    ) -> Self {
        let choices = choices.into();
        assert!(choices.len() < u8::MAX as usize);
        assert!(default_index < (choices.len() - 1) as u8);
        Self {
            value: AtomicU8::new(default_index),
            name: name.into(),
            default: default_index,
            choices,
        }
    }

    pub fn index(&self) -> u8 {
        self.value.load(Ordering::Relaxed)
    }

    pub fn set_index(&self, val: u8) {
        self.value.store(
            val.clamp(0, (self.choices.len() - 1) as u8),
            Ordering::Relaxed,
        );
    }

    pub fn choice(&self) -> &str {
        &self.choices[self.index() as usize]
    }

    pub fn set_choice(&self, choice: &str) -> Result<(), ()> {
        for (i, c) in self.choices.iter().enumerate() {
            if c == choice {
                self.set_index(i as u8);
                return Ok(());
            }
        }
        Err(())
    }

    pub fn choices(&self) -> &[String] {
        &self.choices
    }

    pub fn default_index(&self) -> u8 {
        self.default
    }

    pub fn default_choice(&self) -> &str {
        &self.choices[self.default as usize]
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn atomic(&self) -> &AtomicU8 {
        &self.value
    }
}

pub struct IntParameter {
    value: AtomicI32,
    name: String,
    default: i32,
    range: RangeInclusive<i32>,
}

impl IntParameter {
    pub fn new(name: impl Into<String>, default: i32, range: RangeInclusive<i32>) -> Self {
        assert!(range.contains(&default));
        Self {
            value: AtomicI32::new(default),
            name: name.into(),
            default: default,
            range,
        }
    }

    pub fn value(&self) -> i32 {
        self.value.load(Ordering::Relaxed)
    }

    pub fn set_value(&self, val: i32) {
        self.value.store(
            val.clamp(*self.range.start(), *self.range.end()),
            Ordering::Relaxed,
        );
    }

    pub fn range(&self) -> &RangeInclusive<i32> {
        &self.range
    }

    pub fn default(&self) -> i32 {
        self.default
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn atomic(&self) -> &AtomicI32 {
        &self.value
    }
}

pub struct BoolParameter {
    value: AtomicBool,
    name: String,
    default: bool,
}

impl BoolParameter {
    pub fn new(name: impl Into<String>, default: bool) -> Self {
        Self {
            value: AtomicBool::new(default),
            name: name.into(),
            default,
        }
    }

    pub fn value(&self) -> bool {
        self.value.load(Ordering::Relaxed)
    }

    pub fn set_value(&self, val: bool) {
        self.value.store(val, Ordering::Relaxed);
    }

    pub fn default(&self) -> bool {
        self.default
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn atomic(&self) -> &AtomicBool {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_float() {
        let param = FloatParameter::new("test-param", 1.0, 0.0..=10.0);

        assert_eq!(param.default(), 1.0);
        assert_eq!(param.name(), "test-param");
        assert_eq!(param.range().clone(), 0.0..=10.0);
        assert_eq!(param.value(), 1.0);

        param.set_value(5.0);
        assert_eq!(param.value(), 5.0);

        param.set_value(15.0);
        assert_eq!(param.value(), 10.0);

        param.set_value(-10.0);
        assert_eq!(param.value(), 0.0);
    }

    #[test]
    #[should_panic]
    fn test_float_panic() {
        let _ = FloatParameter::new("test-param", 11.0, 0.0..=10.0);
    }

    #[test]
    #[should_panic]
    fn test_float_panic2() {
        let _ = FloatParameter::new("test-param", -1.0, 0.0..=10.0);
    }

    #[test]
    fn test_choice() {
        let param = ChoiceParameter::new(
            "test-param",
            1,
            &["None".to_string(), "One".to_string(), "Many".to_string()],
        );

        assert_eq!(param.default_index(), 1);
        assert_eq!(param.default_choice(), "One");
        assert_eq!(param.name(), "test-param");
        assert_eq!(
            param.choices(),
            &["None".to_string(), "One".to_string(), "Many".to_string()]
        );
        assert_eq!(param.index(), 1);
        assert_eq!(param.choice(), "One");

        param.set_index(0);
        assert_eq!(param.index(), 0);
        assert_eq!(param.choice(), "None");

        param.set_index(15);
        assert_eq!(param.index(), 2);
        assert_eq!(param.choice(), "Many");
    }

    #[test]
    #[should_panic]
    fn test_choice_panic() {
        let _ = ChoiceParameter::new(
            "test-param",
            3,
            &["None".to_string(), "One".to_string(), "Many".to_string()],
        );
    }

    #[test]
    fn test_int() {
        let param = IntParameter::new("test-param", -5, -10..=10);

        assert_eq!(param.default(), -5);
        assert_eq!(param.name(), "test-param");
        assert_eq!(param.range().clone(), -10..=10);
        assert_eq!(param.value(), -5);

        param.set_value(5);
        assert_eq!(param.value(), 5);

        param.set_value(15);
        assert_eq!(param.value(), 10);

        param.set_value(-15);
        assert_eq!(param.value(), -10);
    }

    #[test]
    #[should_panic]
    fn test_int_panic() {
        let _ = IntParameter::new("test-param", 11, -10..=10);
    }

    #[test]
    #[should_panic]
    fn test_int_panic2() {
        let _ = IntParameter::new("test-param", -11, 0..=10);
    }

    #[test]
    fn test_bool() {
        let param = BoolParameter::new("test-param", true);

        assert_eq!(param.default(), true);
        assert_eq!(param.name(), "test-param");
        assert_eq!(param.value(), true);

        param.set_value(false);
        assert_eq!(param.value(), false);

        param.set_value(true);
        assert_eq!(param.value(), true);
    }
}
