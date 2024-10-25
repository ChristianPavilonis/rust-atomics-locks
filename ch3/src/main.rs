use std::{
    sync::atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering::*},
    thread::{self, sleep},
    time::Duration,
};

fn main() {
    // happens_before();
    // acquire_release();
    // acquire_release_unsafe();
    // locking();
    // seqcst();
    fence();
}

static FOO: AtomicI32 = AtomicI32::new(0);

fn happens_before() {
    FOO.store(1, Relaxed);
    // spawning a thread creates a happens-before relationship
    // with the first FOO.store()
    // so FOO will always be 1
    let t = thread::spawn(foo);
    // however there's no happens before between foo() and store(2)
    FOO.store(2, Relaxed);
    // t.join creates a happens before so x will never be 3 before foo() is called
    t.join().unwrap();
    FOO.store(3, Relaxed);
}

fn foo() {
    let x = FOO.load(Relaxed);
    // prints 2 most of the time, but some times 1
    println!("{x}");
    assert!(x == 1 || x == 2);
}

fn acquire_release() {
    static DATA: AtomicU64 = AtomicU64::new(0);
    static READY: AtomicBool = AtomicBool::new(false);

    thread::spawn(|| {
        DATA.store(123, Relaxed);
        READY.store(true, Release);
    });

    while !READY.load(Acquire) {
        thread::sleep(Duration::from_millis(100));
        println!("waiting...")
    }

    println!("{}", DATA.load(Relaxed));
}

fn acquire_release_unsafe() {
    static mut DATA: u64 = 0;
    static READY: AtomicBool = AtomicBool::new(false);

    thread::spawn(|| {
        unsafe { DATA = 1234 };
        READY.store(true, Release);
    });

    while !READY.load(Acquire) {
        thread::sleep(Duration::from_millis(100));
        println!("waiting...")
    }

    println!("{}", unsafe { DATA });
}

static mut LOCK_DATA: String = String::new();
static LOCKED: AtomicBool = AtomicBool::new(false);

fn f() {
    if LOCKED
        .compare_exchange(false, true, Acquire, Relaxed)
        .is_ok()
    {
        // we hold the exclusive lock, so nothing else is accessing LOCK_DATA
        unsafe { LOCK_DATA.push('!') };
        LOCKED.store(false, Release);
    }
}

fn locking() {
    thread::scope(|s| {
        for _ in 0..100 {
            s.spawn(f);
        }
    });

    sleep(Duration::from_secs(1));

    if LOCKED.swap(true, Acquire) == false {
        // this is the same as the compare_exchange above
        unsafe {
            println!("{}", LOCK_DATA.len());
        }
    }
}

static A: AtomicBool = AtomicBool::new(false);
static B: AtomicBool = AtomicBool::new(false);

static mut S: String = String::new();

fn seqcst() {
    let a = thread::spawn(|| {
        A.store(true, SeqCst);
        if !B.load(SeqCst) {
            unsafe { S.push('!') };
        }
    });

    let b = thread::spawn(|| {
        B.store(true, SeqCst);
        if !A.load(SeqCst) {
            unsafe { S.push('!') };
        }
    });

    a.join().unwrap();
    b.join().unwrap();

    unsafe {
        println!("{}", S);
    }
}

