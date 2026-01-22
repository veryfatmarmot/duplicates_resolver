use std::time::Instant;

pub struct ScopeTimeLogger {
    name: String,
    start: Instant,
}

impl ScopeTimeLogger {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
        }
    }
}

impl Drop for ScopeTimeLogger {
    fn drop(&mut self) {
        let time = Instant::now() - self.start;
        println!("'{}' block took {:?}.", self.name, time);
    }
}
