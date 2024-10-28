use std::{cell::UnsafeCell, mem::MaybeUninit, sync::{atomic::{AtomicBool, Ordering},  Arc}};


pub fn channel<T>() -> (Sender<T>, Receiver<T>){
    let a = Arc::new(Channel {
        message: UnsafeCell::new(MaybeUninit::uninit()),
        ready: AtomicBool::new(false),
    });

    (Sender {channel: a.clone()}, Receiver { channel: a})
}

pub struct Sender<T> {
    channel: Arc<Channel<T>>,
}

impl<T> Sender<T> {
    pub fn send(self, message: T) {
        unsafe { (*self.channel.message.get()).write(message)};
        self.channel.ready.store(true, Ordering::Release)
    }
}

pub struct  Receiver<T> {
    channel: Arc<Channel<T>>,
}

impl<T> Receiver<T> {
    pub fn is_ready(&self) -> bool {
        self.channel.ready.load(Ordering::Relaxed)
    }

    pub fn receive(self) -> T {
        if !self.channel.ready.swap(false, Ordering::Acquire) {
            panic!("no message available!");
        }
        unsafe { (*self.channel.message.get()).assume_init_read() }
    }
}

struct Channel<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    ready: AtomicBool,
}

unsafe impl<T> Sync for Channel<T> where T: Send {}

impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        if *self.ready.get_mut() {
            unsafe { self.message.get_mut().assume_init_drop() }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use super::*;

    #[test]
    fn it_works() {
        thread::scope(|s| {
            let (sender, receiver) = channel();
            let t = thread::current();

            s.spawn(move || {
                sender.send("hello, world!");
                // sender.send("foo"); // compile error! use after move.
                t.unpark();
            });

            while !receiver.is_ready() {
                thread::park();
            }
            assert_eq!(receiver.receive(), "hello, world!");
        });
    }
}
