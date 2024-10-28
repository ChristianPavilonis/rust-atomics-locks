use std::{
    collections::VecDeque,
    sync::{Condvar, Mutex},
};

mod one_shot;

fn main() {
    println!("Hello, world!");
}

pub struct Channel<T> {
    queue: Mutex<VecDeque<T>>,
    item_ready: Condvar,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            item_ready: Condvar::new(),
        }
    }

    pub fn send(&self, message: T) {
        self.queue.lock().unwrap().push_back(message);
        self.item_ready.notify_one();
    }

    pub fn receive(&self) -> T {
        let mut b = self.queue.lock().unwrap();

        loop {
            if let Some(message) = b.pop_front() {
                return message;
            }
            b = self.item_ready.wait(b).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        thread::{self, sleep},
        time::Duration,
    };

    use crate::Channel;

    #[test]
    fn test_channel() {
        let channel: Channel<&str> = Channel::new();

        thread::scope(|s| {
            s.spawn(|| {
                let message = channel.receive();
                assert_eq!("I hand grind", message);

                let message = channel.receive();
                assert_eq!("every morning", message);
            });
            s.spawn(|| {
                channel.send("I hand grind");
                channel.send("every morning");
            });
        });
    }
}
