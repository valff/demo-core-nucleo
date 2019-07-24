//! Push-button thread.

use crate::thr::{self, led::FAST};
use core::sync::atomic::{AtomicU32, Ordering::*};
use drone_core::awt_for;
use drone_cortex_m::{fib, reg::prelude::*, thr::prelude::*};
use drone_stm32_map::periph::exti::{Exti13, ExtiPeriph};
use futures::prelude::*;

const DEBOUNCE_INTERVAL: u32 = 2000;

static DEBOUNCE: AtomicU32 = AtomicU32::new(0);

/// The thread input.
#[allow(missing_docs)]
pub struct Input {
    pub exti13: ExtiPeriph<Exti13>,
    pub exti15_10: thr::Exti1510<Att>,
    pub sys_tick: thr::SysTick<Att>,
}

/// The thread handler.
pub fn handler(input: Input) -> impl Future<Output = !> {
    let Input {
        exti13,
        exti15_10,
        sys_tick,
    } = input;
    asnc(static move || {
        if false {
            yield;
        }
        sys_tick.add(|| {
            loop {
                let debounce = DEBOUNCE.load(Relaxed);
                if debounce != 0 {
                    DEBOUNCE.store(debounce - 1, Relaxed);
                }
                yield;
            }
        });
        let stream = exti15_10.add_stream_skip(fib::new(move || {
            loop {
                if exti13.exti_pr_pif.read_bit() {
                    exti13.exti_pr_pif.set_bit();
                    yield Some(());
                }
                yield None;
            }
        }));
        awt_for!(() in stream => {
            if DEBOUNCE.load(Relaxed) == 0 {
                FAST.store(!FAST.load(Relaxed), Relaxed);
                DEBOUNCE.store(DEBOUNCE_INTERVAL, Relaxed);
            }
        });
        unreachable!()
    })
}
