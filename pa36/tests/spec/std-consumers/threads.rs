// Spawned threads, scoped threads, channels and per-thread storage. Results are
// joined or sorted before printing, so the output does not depend on scheduling.
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Barrier, Condvar, Mutex, RwLock};
use std::thread;
use std::time::Duration;

thread_local! {
    static LOCAL: Cell<u64> = const { Cell::new(0) };
}

fn is_prime(n: u64) -> bool {
    n >= 2 && (2..).take_while(|d| d * d <= n).all(|d| n % d != 0)
}

fn main() {
    let handles: Vec<thread::JoinHandle<(usize, usize)>> = (0..4)
        .map(|worker| {
            thread::spawn(move || {
                let start = worker as u64 * 5000;
                (worker, (start..start + 5000).filter(|n| is_prime(*n)).count())
            })
        })
        .collect();
    let counts: Vec<(usize, usize)> = handles.into_iter().map(|handle| handle.join().unwrap()).collect();
    println!("spawned {:?} total {}", counts, counts.iter().map(|(_, count)| count).sum::<usize>());

    let counter = Arc::new(AtomicUsize::new(0));
    let total = Arc::new(Mutex::new(0u64));
    let workers: Vec<_> = (1..=8u64)
        .map(|worker| {
            let counter = Arc::clone(&counter);
            let total = Arc::clone(&total);
            thread::spawn(move || {
                for step in 0..1000 {
                    counter.fetch_add(1, Ordering::Relaxed);
                    *total.lock().unwrap() += worker * step;
                }
            })
        })
        .collect();
    for worker in workers {
        worker.join().unwrap();
    }
    println!("shared {} {} {}", counter.load(Ordering::SeqCst), total.lock().unwrap(), Arc::strong_count(&counter));

    let data: Vec<u32> = (1..=1000).collect();
    let mut partial = [0u64; 4];
    thread::scope(|scope| {
        for (chunk, slot) in data.chunks(250).zip(partial.iter_mut()) {
            scope.spawn(move || *slot = chunk.iter().map(|v| u64::from(*v) * u64::from(*v)).sum());
        }
    });
    let largest = thread::scope(|scope| scope.spawn(|| data.iter().copied().max()).join().unwrap());
    println!("scoped {:?} {} {:?}", partial, partial.iter().sum::<u64>(), largest);

    let (sender, receiver) = mpsc::channel();
    for producer in 0..3u32 {
        let sender = sender.clone();
        thread::spawn(move || {
            for item in 0..5 {
                sender.send(producer * 100 + item).unwrap();
            }
        });
    }
    drop(sender);
    let mut received: Vec<u32> = receiver.iter().collect();
    received.sort_unstable();
    println!("channel {} {:?} {:?}", received.len(), &received[..6], received.last());

    let (request, requests) = mpsc::sync_channel::<u64>(0);
    let (reply, replies) = mpsc::channel();
    let server = thread::spawn(move || {
        let mut served = 0;
        for value in requests {
            reply.send(value * value).unwrap();
            served += 1;
        }
        served
    });
    let squares: Vec<u64> = (1..=5).map(|value| {
        request.send(value).unwrap();
        replies.recv().unwrap()
    }).collect();
    drop(request);
    println!("rendezvous {:?} served {}", squares, server.join().unwrap());

    let locals: Vec<u64> = (1..=3u64)
        .map(|worker| {
            thread::spawn(move || {
                for _ in 0..worker * 10 {
                    LOCAL.set(LOCAL.get() + worker);
                }
                LOCAL.get()
            })
        })
        .map(|handle| handle.join().unwrap())
        .collect();
    println!("thread locals {:?} main {}", locals, LOCAL.get());

    let named = thread::Builder::new().name("worker-seven".to_string()).spawn(|| thread::current().name().map(str::to_owned)).unwrap();
    let main_id = thread::current().id();
    let other_id = thread::spawn(|| thread::current().id()).join().unwrap();
    println!("named {:?} main {:?} distinct ids {}", named.join().unwrap(), thread::current().name(), main_id != other_id);

    let ready = Arc::new((Mutex::new(false), Condvar::new()));
    let waiter = {
        let ready = Arc::clone(&ready);
        thread::spawn(move || {
            let (flag, signal) = &*ready;
            let mut guard = flag.lock().unwrap();
            while !*guard {
                guard = signal.wait(guard).unwrap();
            }
            "signalled"
        })
    };
    thread::sleep(Duration::from_millis(5));
    *ready.0.lock().unwrap() = true;
    ready.1.notify_all();
    println!("condvar {}", waiter.join().unwrap());

    let barrier = Arc::new(Barrier::new(4));
    let waiting: Vec<thread::JoinHandle<bool>> = (0..4)
        .map(|_| {
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || barrier.wait().is_leader())
        })
        .collect();
    let leaders: usize = waiting.into_iter().map(|handle| usize::from(handle.join().unwrap())).sum();
    let cache = Arc::new(RwLock::new(vec![0u32; 8]));
    let writers: Vec<_> = (0..8)
        .map(|index| {
            let cache = Arc::clone(&cache);
            thread::spawn(move || {
                let seen = cache.read().unwrap().len();
                cache.write().unwrap()[index] = (index * seen) as u32;
            })
        })
        .collect();
    for writer in writers {
        writer.join().unwrap();
    }
    println!("barrier leaders {} rwlock {:?}", leaders, cache.read().unwrap());
    println!("parallelism known {}", thread::available_parallelism().map(|n| n.get() >= 1).unwrap_or(true));
    thread::yield_now();
    let finished = thread::spawn(|| 7).join();
    println!("joined {:?}", finished.ok());
}
