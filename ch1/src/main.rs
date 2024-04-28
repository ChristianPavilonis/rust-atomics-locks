use std::{
    cell::Cell,
    collections::VecDeque,
    sync::{Condvar, Mutex},
    thread,
    time::Duration,
    usize,
};

fn main() {
    println!("Example threads -----");
    example1();
    println!("---------------------\n");

    println!("Thread with Result --");
    thread_with_result();
    println!("---------------------\n");

    println!("Scoped threads ------");
    scoped_threads();
    println!("---------------------\n");

    println!("Shared ownership ----");
    shared_ownership();
    println!("---------------------\n");

    // println!("Thread Parking ------");
    // thread_parking();
    // println!("---------------------\n");

    // println!("Condition Varialbes -");
    // condition_variables();
    // println!("---------------------\n");
}

fn example1() {
    let t1 = thread::spawn(f);
    let t2 = thread::spawn(f);

    println!("Hello from the main thread");

    t1.join().unwrap();
    t2.join().unwrap();
}

fn f() {
    println!("hello from another tread");

    let id = thread::current().id();
    println!("this is thread {:?}", id);
}

fn thread_with_result() {
    let numbers = Vec::from_iter(0..=1000);

    let t = thread::spawn(move || {
        let len = numbers.len();
        let sum = numbers.iter().sum::<usize>();
        sum / len
    })
    .join()
    .unwrap();

    println!("avg: {}", t);
}

fn scoped_threads() {
    let numbers = vec![1, 2, 3];

    thread::scope(|s| {
        s.spawn(|| {
            println!("len: {}", numbers.len());
        });
        s.spawn(|| {
            for n in &numbers {
                println!("num: {}", n);
            }
        });
    });
    // compiler error if trying to use as mutable in two threads
    // let mut numbers = vec![1, 2, 3];
    //
    // thread::scope(|s| {
    //     s.spawn(|| {
    //         numbers.push(4);
    //     });
    //     s.spawn(|| {
    //         numbers.push(5);
    //     });
    // });
}

static X: [i32; 3] = [1, 2, 3];

fn shared_ownership() {
    // use statics for sharing
    thread::spawn(|| dbg!(&X)).join().unwrap();
    thread::spawn(|| dbg!(&X)).join().unwrap();

    // use Box::leak for sharing
    let x: &'static [i32; 3] = Box::leak(Box::new([1, 2, 3]));

    thread::spawn(move || dbg!(x)).join().unwrap();
    thread::spawn(move || dbg!(x)).join().unwrap();

    use std::sync::Arc;

    let a = Arc::new([7, 8, 9]);
    let b = a.clone();

    thread::spawn(move || dbg!(a)).join().unwrap();
    thread::spawn(move || dbg!(b)).join().unwrap();
}

fn thread_parking() {
    let queue = Mutex::new(VecDeque::new());

    thread::scope(|s| {
        let t = s.spawn(|| loop {
            let item = queue.lock().unwrap().pop_front();
            if let Some(item) = item {
                dbg!(item);
            } else {
                thread::park();
            }
        });

        for i in 0..3 {
            queue.lock().unwrap().push_back(i);
            t.thread().unpark();
            thread::sleep(Duration::from_secs(1));
        }
    });
}

fn condition_variables() {
    let queue = Mutex::new(VecDeque::new());
    let not_empty = Condvar::new();

    thread::scope(|s| {
        s.spawn(|| loop {
            let mut q = queue.lock().unwrap();
            let item = loop {
                if let Some(item) = q.pop_front() {
                    break item;
                } else {
                    q = not_empty.wait(q).unwrap();
                }
            };
            drop(q);
            dbg!(item);
        });

        for i in 0..3 {
            queue.lock().unwrap().push_back(i);
            not_empty.notify_one();
            thread::sleep(Duration::from_secs(1));
        }
        std::process::exit(0);
    });
}

#[cfg(test)]
mod tests {
    use std::{
        borrow::BorrowMut,
        cell::{Cell, RefCell},
        collections::VecDeque,
        sync::Mutex,
        thread,
        time::Duration,
    };

    #[test]
    fn cells() {
        let v = Cell::new(vec![1, 2, 3]);
        let mut v2 = v.take();

        // v is now empty
        assert_eq!(v2.len(), 3);

        v2.push(4);
        v.set(v2);

        assert_eq!(v.into_inner().len(), 4);
    }
    #[test]
    fn refcells() {
        let v = RefCell::new(vec![1, 2, 3]);
        v.borrow_mut().push(4);

        assert_eq!(v.into_inner().len(), 4);
    }

    #[test]
    fn mutexes() {
        let n = Mutex::new(0);

        thread::scope(|s| {
            for _ in 0..10 {
                s.spawn(|| {
                    let mut guard = n.lock().unwrap();
                    for _ in 0..100 {
                        *guard += 1;
                    }
                    drop(guard);
                    thread::sleep(Duration::from_secs(1));
                });
            }
        });

        assert_eq!(n.into_inner().unwrap(), 1000);
    }
}
