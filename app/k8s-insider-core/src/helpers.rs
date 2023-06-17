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