use std::{time::{SystemTime, Duration}, any::type_name};

pub fn get_secs_since_unix_epoch() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(2137420))
        .as_secs()
}

pub fn pretty_type_name<'a, T>() -> &'a str {
    type_name::<T>().split("::").last().unwrap()
}

pub trait ApplyConditional<F: FnOnce(Self) -> Self>
where Self: Sized {
    fn and_if(self, condition: bool, func: F) -> Self;
}

impl<T, F: FnOnce(Self) -> Self> ApplyConditional<F> for T {
    fn and_if(self, condition: bool, func: F) -> Self {
        let mut obj = self;
        if condition {
            obj = func(obj);
        }

        obj
    }
}

pub trait ApplyConditionalMut<F: FnOnce(&mut Self) -> &mut Self> {
    fn with_condition(&mut self, condition: bool, func: F) -> &mut Self;
}

impl<T, F: FnOnce(&mut Self) -> &mut Self> ApplyConditionalMut<F> for T {
    fn with_condition(&mut self, condition: bool, func: F) -> &mut Self {
        if condition {
            func(self);
        }

        self
    }
}