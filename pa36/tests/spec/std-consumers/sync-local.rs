// Synchronisation primitives and thread-local storage used from the main thread only.
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, LazyLock, Mutex, Once, OnceLock, RwLock};
use std::thread;

thread_local! {
    static COUNTER: Cell<u32> = const { Cell::new(0) };
    static NAMES: RefCell<Vec<String>> = RefCell::new(Vec::new());
    static LAZY: String = {
        INITIALISED.fetch_add(1, Ordering::SeqCst);
        String::from("initialised on first use")
    };
}

static INITIALISED: AtomicUsize = AtomicUsize::new(0);
static GLOBAL: Mutex<Vec<u32>> = Mutex::new(Vec::new());
static CONFIG: OnceLock<HashMap<&'static str, u32>> = OnceLock::new();
static TABLE: LazyLock<Vec<u64>> = LazyLock::new(|| {
    INITIALISED.fetch_add(10, Ordering::SeqCst);
    (0..10).map(|n| n * n).collect()
});
static START: Once = Once::new();
static IDS: AtomicU64 = AtomicU64::new(1);

fn next_id() -> u64 {
    IDS.fetch_add(1, Ordering::Relaxed)
}

fn config() -> &'static HashMap<&'static str, u32> {
    CONFIG.get_or_init(|| {
        INITIALISED.fetch_add(100, Ordering::SeqCst);
        HashMap::from([("jobs", 4), ("retries", 2)])
    })
}

fn bump() -> u32 {
    COUNTER.with(|counter| {
        counter.set(counter.get() + 1);
        counter.get()
    })
}

fn main() {
    println!("thread {:?} {}", thread::current().name(), thread::current().id() == thread::current().id());
    println!("local {} {} {} {}", bump(), bump(), COUNTER.get(), COUNTER.replace(40));
    COUNTER.set(COUNTER.get() + 2);
    NAMES.with_borrow_mut(|names| names.push("first".to_string()));
    NAMES.with(|names| names.borrow_mut().push("second".to_string()));
    println!("local {} {:?} {}", COUNTER.take(), NAMES.with_borrow(|names| names.join("+")), NAMES.take().len());
    println!("lazy local {} {} {}", INITIALISED.load(Ordering::SeqCst), LAZY.with(|text| text.len()), INITIALISED.load(Ordering::SeqCst));
    LAZY.with(|text| println!("lazy local {} {}", text, INITIALISED.load(Ordering::SeqCst)));

    let mutex = Mutex::new(vec![1, 2, 3]);
    {
        let mut guard = mutex.lock().unwrap();
        guard.push(4);
        println!("mutex {:?} {} {}", *guard, mutex.try_lock().is_err(), mutex.is_poisoned());
    }
    println!("mutex {:?} {:?}", mutex.try_lock().map(|guard| guard.len()).ok(), mutex);
    *mutex.lock().unwrap() = vec![9];
    println!("mutex {:?}", mutex.into_inner().unwrap());
    GLOBAL.lock().unwrap().extend([3, 1, 2]);
    GLOBAL.lock().unwrap().sort();
    println!("global {:?}", GLOBAL.lock().unwrap());

    let lock = RwLock::new(String::from("shared"));
    {
        let first = lock.read().unwrap();
        let second = lock.read().unwrap();
        println!("rwlock {} {} {}", first.len() + second.len(), lock.try_write().is_err(), lock.try_read().is_ok());
    }
    lock.write().unwrap().push_str(" and written");
    println!("rwlock {:?} {}", lock.read().unwrap(), lock.try_write().is_ok());
    println!("rwlock {:?}", lock.into_inner().unwrap());

    println!("once lock {:?} {}", CONFIG.get().is_none(), INITIALISED.load(Ordering::SeqCst));
    println!("once lock {:?} {:?} {}", config().get("jobs"), config().get("missing"), INITIALISED.load(Ordering::SeqCst));
    println!("once lock {:?} {}", CONFIG.set(HashMap::new()).is_err(), config().len());
    let local: OnceLock<u8> = OnceLock::new();
    println!("once lock {:?} {:?} {:?} {:?}", local.get(), local.set(5), local.set(6), local.get());
    println!("lazy lock {} {} {}", INITIALISED.load(Ordering::SeqCst), TABLE[9], INITIALISED.load(Ordering::SeqCst));
    println!("lazy lock {:?} {}", &TABLE[..4], TABLE.iter().sum::<u64>());
    let mut calls = 0;
    START.call_once(|| calls += 1);
    START.call_once(|| calls += 1);
    println!("once {} {}", calls, START.is_completed());
    println!("ids {} {} {} {}", next_id(), next_id(), next_id(), IDS.load(Ordering::SeqCst));

    let shared = Arc::new(Mutex::new(HashMap::new()));
    let handles: Vec<Arc<Mutex<HashMap<u32, String>>>> = (0..3).map(|_| Arc::clone(&shared)).collect();
    for (index, handle) in handles.iter().enumerate() {
        handle.lock().unwrap().insert(index as u32, format!("entry {index}"));
    }
    let mut entries: Vec<(u32, String)> = shared.lock().unwrap().drain().collect();
    entries.sort();
    println!("arc mutex {:?} {}", entries, Arc::strong_count(&shared));

    let (sender, receiver) = mpsc::channel();
    let second = sender.clone();
    for value in 0..3 {
        sender.send(value).unwrap();
        second.send(value * 10).unwrap();
    }
    println!("channel {:?} {:?}", receiver.recv(), receiver.try_recv());
    drop(sender);
    drop(second);
    let rest: Vec<i32> = receiver.iter().collect();
    println!("channel {:?} {:?}", rest, receiver.try_recv().is_err());
    let (bounded, bounded_receiver) = mpsc::sync_channel(2);
    println!("bounded {:?} {:?} {:?}", bounded.try_send('a').is_ok(), bounded.try_send('b').is_ok(), bounded.try_send('c').is_err());
    println!("bounded {:?} {:?}", bounded_receiver.recv(), bounded_receiver.try_iter().collect::<String>());
    drop(bounded_receiver);
    println!("disconnected {:?}", bounded.send('z').is_err());
}
