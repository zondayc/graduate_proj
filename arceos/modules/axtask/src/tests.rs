use core::sync::atomic::{AtomicUsize, Ordering};
use spin::{Mutex, Once};

use crate as axtask;

static INIT: Once<()> = Once::new();
static SERIAL: Mutex<()> = Mutex::new(());

#[test]
fn test_sched_fifo() {
    let _lock = SERIAL.lock();
    INIT.call_once(|| axtask::init_scheduler());

    const NUM_TASKS: usize = 10;
    static FINISHED_TASKS: AtomicUsize = AtomicUsize::new(0);

    for i in 0..NUM_TASKS {
        axtask::spawn(move || {
            println!("Hello, task {}! id = {:?}", i, axtask::current().id());
            axtask::yield_now();
            let order = FINISHED_TASKS.fetch_add(1, Ordering::Relaxed);
            assert_eq!(order, i); // FIFO scheduler
        });
    }
    while FINISHED_TASKS.load(Ordering::Relaxed) < NUM_TASKS {
        axtask::yield_now();
    }
}

#[test]
fn test_fp_state_switch() {
    let _lock = SERIAL.lock();
    INIT.call_once(|| axtask::init_scheduler());

    const NUM_TASKS: usize = 5;
    const FLOATS: [f64; NUM_TASKS] = [
        3.141592653589793,
        2.718281828459045,
        -1.4142135623730951,
        0.0,
        0.618033988749895,
    ];
    static FINISHED_TASKS: AtomicUsize = AtomicUsize::new(0);

    for i in 0..NUM_TASKS {
        axtask::spawn(move || {
            let mut value = FLOATS[i] + i as f64;
            axtask::yield_now();
            value -= i as f64;

            println!("Float {} = {}", i, value);
            assert!((value - FLOATS[i]).abs() < 1e-9);
            FINISHED_TASKS.fetch_add(1, Ordering::Relaxed);
        });
    }
    while FINISHED_TASKS.load(Ordering::Relaxed) < NUM_TASKS {
        axtask::yield_now();
    }
}
