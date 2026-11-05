use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};

static mut TRACE: u64 = 0;
fn mark(value: u64) { unsafe { TRACE = TRACE * 10 + value; } }
fn reset() { unsafe { TRACE = 0; } }
fn trace() -> u64 { unsafe { TRACE } }
struct Mark(u64);
impl Drop for Mark { fn drop(&mut self) { mark(self.0); } }
fn argument(value: u64, effect: u64) -> Mark { mark(effect); Mark(value) }
fn poll<F: Future>(future: Pin<&mut F>) -> Poll<F::Output> {
    let mut context = Context::from_waker(Waker::noop());
    Future::poll(future, &mut context)
}
async fn work(_a: Mark, _b: Mark) -> Mark {
    mark(5);
    let _local = Mark(6);
    mark(7);
    Mark(8)
}
fn unpolled() -> u64 {
    reset();
    { let _future = work(argument(3, 1), argument(4, 2)); mark(9); }
    mark(9);
    trace()
}
fn completed() -> u64 {
    reset();
    let output;
    {
        let mut future = work(argument(3, 1), argument(4, 2));
        mark(9);
        output = match poll(unsafe { Pin::new_unchecked(&mut future) }) {
            Poll::Ready(value) => value,
            Poll::Pending => return 0,
        };
        mark(9);
        // The pinned future stays in this storage until its implicit in-place Drop.
    }
    mark(9);
    drop(output);
    mark(9);
    trace()
}
async fn pattern((a, _): (Mark, Mark), _: Mark) -> u32 {
    mark(7);
    drop(a);
    42
}
fn pattern_unpolled() -> u64 {
    reset();
    { let _future = pattern((argument(3, 1), argument(4, 2)), argument(6, 5)); mark(9); }
    mark(9);
    trace()
}
fn pattern_completed() -> u64 {
    reset();
    {
        let mut future = pattern((argument(3, 1), argument(4, 2)), argument(6, 5));
        mark(9);
        match poll(unsafe { Pin::new_unchecked(&mut future) }) {
            Poll::Ready(42) => {},
            _ => return 0,
        }
        mark(9);
    }
    mark(9);
    trace()
}
async fn passthrough(value: Mark) -> Mark { mark(5); value }
fn returned_capture() -> u64 {
    reset();
    let output;
    {
        let mut future = passthrough(argument(3, 1));
        mark(9);
        output = match poll(unsafe { Pin::new_unchecked(&mut future) }) {
            Poll::Ready(value) => value,
            Poll::Pending => return 0,
        };
        mark(9);
    }
    mark(9);
    drop(output);
    mark(9);
    trace()
}
async fn bump(value: &mut u32) -> u32 { mark(2); *value += 1; *value }
fn borrowed_capture() -> u64 {
    reset();
    let mut value = 7;
    { let _future = bump(&mut value); mark(1); }
    mark(u64::from(value));
    {
        let mut future = bump(&mut value);
        mark(1);
        match poll(unsafe { Pin::new_unchecked(&mut future) }) {
            Poll::Ready(8) => mark(8),
            _ => return 0,
        }
    }
    mark(u64::from(value));
    trace()
}
struct Wait { state: u8, _guard: Mark }
impl Future for Wait {
    type Output = u32;
    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<u32> {
        mark(3);
        let this = self.get_mut();
        if this.state == 0 {
            this.state = 1;
            context.waker().wake_by_ref();
            Poll::Pending
        } else { Poll::Ready(4) }
    }
}
async fn suspended(_input: Mark) -> u32 {
    let _local = Mark(6);
    Wait { state: 0, _guard: Mark(7) }.await
}
fn cancelled_pending() -> u64 {
    reset();
    {
        let mut future = suspended(argument(2, 1));
        mark(9);
        if !matches!(poll(unsafe { Pin::new_unchecked(&mut future) }), Poll::Pending) { return 0; }
        mark(9);
    }
    mark(9);
    trace()
}
fn completed_pending() -> u64 {
    reset();
    {
        let mut future = suspended(argument(2, 1));
        mark(9);
        if !matches!(poll(unsafe { Pin::new_unchecked(&mut future) }), Poll::Pending) { return 0; }
        mark(9);
        if !matches!(poll(unsafe { Pin::new_unchecked(&mut future) }), Poll::Ready(4)) { return 0; }
        mark(9);
    }
    mark(9);
    trace()
}
fn main() {
    println!("unpolled {}", unpolled());
    println!("completed {}", completed());
    println!("pattern-unpolled {}", pattern_unpolled());
    println!("pattern-completed {}", pattern_completed());
    println!("returned-capture {}", returned_capture());
    println!("borrowed-capture {}", borrowed_capture());
    println!("cancelled-pending {}", cancelled_pending());
    println!("completed-pending {}", completed_pending());
}
