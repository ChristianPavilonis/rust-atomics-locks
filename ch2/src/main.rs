use std::sync::atomic::{AtomicU32, AtomicU64};

use std::thread::current;
use std::time::Instant;
use std::{
    sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed},
    thread,
    time::Duration,
};

fn main() {
    // stop_flag();
    // progress();

    // lazy initialization
    // println!("{}", lazy_init());
    // println!("{}", lazy_init());

    // multi_threaded_with_stats();

    // id_allocation();

    // compair_exchange
    // let a = AtomicU32::new(0);
    // compair_exchange(&a);
    // compair_exchange(&a);
    // println!("{}", a.load(Relaxed));

    // compair_exchange_weak
    println!("{}", allocate_new_id_weak());
    println!("{}", allocate_new_id_weak());

    for _ in 0..1000 {
        allocate_new_id_weak();
    }
}

fn stop_flag() {
    static STOP: AtomicBool = AtomicBool::new(false);

    let background_thread = thread::spawn(|| {
        while !STOP.load(Relaxed) {
            println!("tell me to stop");
            thread::sleep(Duration::from_millis(500));
        }
    });

    for line in std::io::stdin().lines() {
        match line.unwrap().as_str() {
            "help" => println!("commands: help, stop"),
            "stop" => break,
            cmd => println!("unknown command: {cmd:?}"),
        }
    }

    STOP.store(true, Relaxed);

    background_thread.join().unwrap();
}

fn progress() {
    let num_done = AtomicUsize::new(0);

    let main_thread = thread::current();

    thread::scope(|s| {
        s.spawn(|| {
            for i in 0..100 {
                // do stuff
                num_done.store(i + 1, Relaxed);

                thread::sleep(Duration::from_millis(182));
            }
            main_thread.unpark();
        });

        loop {
            let n = num_done.load(Relaxed);
            if n == 100 {
                break;
            }
            println!("Working... {n}/100 done");

            thread::park_timeout(Duration::from_secs(1));
        }
    });

    println!("donzo");
}

fn multi_threaded_progress() {
    let num_done = &AtomicUsize::new(0);

    thread::scope(|s| {
        for _ in 0..4 {
            s.spawn(move || {
                for _ in 0..25 {
                    // using fetch_add we can atomicly add to the number across all 4 threads
                    // so now we can use 4 threads instead of one background thread.
                    num_done.fetch_add(1, Relaxed);
                    thread::sleep(Duration::from_millis(400));
                }
            });
        }

        loop {
            let n = num_done.load(Relaxed);
            if n == 100 {
                break;
            }
            println!("working.. {n}/100");
            thread::sleep(Duration::from_secs(1));
        }
    });
    println!("done");
}

fn multi_threaded_with_stats() {
    let num_done = &AtomicUsize::new(0);
    let total_time = &AtomicU64::new(0);
    let max_time = &AtomicU64::new(0);

    thread::scope(|s| {
        for _ in 0..4 {
            s.spawn(move || {
                for _ in 0..25 {
                    let start = Instant::now();
                    thread::sleep(Duration::from_millis(200));
                    let time_taken = start.elapsed().as_micros() as u64;
                    num_done.fetch_add(1, Relaxed);
                    total_time.fetch_add(time_taken, Relaxed);
                    max_time.fetch_max(time_taken, Relaxed);
                }
            });
        }

        loop {
            let total_time = Duration::from_micros(total_time.load(Relaxed));
            let max_time = Duration::from_micros(max_time.load(Relaxed));
            let n = num_done.load(Relaxed);
            if n == 100 {
                break;
            }
            if n == 0 {
                continue;
            }
            println!(
                "working.. {n}/100, {:?} average, {:?} peak",
                total_time / n as u32,
                max_time
            );
            thread::sleep(Duration::from_secs(1));
        }
    });
    println!("done");
}

// lazily get's the value x.
// usefull to get a value when it's needed, and save the result for later.
// I wonder how the lazy static crate works? or the once crate?
fn lazy_init() -> u64 {
    static X: AtomicU64 = AtomicU64::new(0);
    let mut x = X.load(Relaxed);
    if x == 0 {
        println!("x is 0 calculating...");
        x = 42; // some computationaly heavy calculation
        X.store(x, Relaxed);
    }
    x
}

// Example using the returned value from a fetch_add

fn id_allocation() {
    println!("{}", allocate_new_id());
    println!("{}", allocate_new_id());
    println!("{}", allocate_new_id());

    for _ in 0..1000 {
        allocate_new_id();
    }
}

fn allocate_new_id() -> u32 {
    // problem is that it could go over u32 max
    // static NEXT_ID: AtomicU32 = AtomicU32::new(0);
    // NEXT_ID.fetch_add(1, Relaxed)

    // wont ever overflow but will eventually get over 1,000
    // static NEXT_ID: AtomicU32 = AtomicU32::new(0);
    // let id = NEXT_ID.fetch_add(1, Relaxed);
    // assert!(id < 1000, "too many ids!");
    // id

    // if we're ever over 1000 then we'll decrement before panicing to make the max 1000.
    static NEXT_ID: AtomicU32 = AtomicU32::new(0);
    let id = NEXT_ID.fetch_add(1, Relaxed);
    if id >= 1000 {
        NEXT_ID.fetch_sub(1, Relaxed);
        panic!("too many ids!");
    }
    id
}

fn compair_exchange(a: &AtomicU32) {
    let mut current = a.load(Relaxed);

    loop {
        let new = current + 1;
        match a.compare_exchange(current, new, Relaxed, Relaxed) {
            Ok(_) => return,
            Err(v) => current = v,
        }
    }
}

// this version will never increment beyond 1000
fn allocate_new_id_weak() -> u32 {
    static NEXT_ID: AtomicU32 = AtomicU32::new(0);
    let mut id = NEXT_ID.load(Relaxed);
    loop {
        assert!(id < 1000, "too many ids!");
        match NEXT_ID.compare_exchange_weak(id, id + 1, Relaxed, Relaxed) {
            Ok(_) => return id,
            Err(v) => id = v,
        }
    }
}
