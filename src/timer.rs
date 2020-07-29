// Copyright 2019 TiKV Project Authors. Licensed under Apache-2.0.

use crate::profiler::perf_signal_handler;
use std::os::raw::c_int;
use std::ptr::null_mut;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;
use winapi::um::winnt::HANDLE;

#[repr(C)]
#[derive(Clone)]
struct Timeval {
    pub tv_sec: i64,
    pub tv_usec: i64,
}

#[repr(C)]
#[derive(Clone)]
struct Itimerval {
    pub it_interval: Timeval,
    pub it_value: Timeval,
}

const ITIMER_PROF: c_int = 2;

pub struct Timer {
    _frequency: c_int,
    receiver: Receiver<()>,
    _cancel_sender: Option<Sender<()>>,
}

impl Timer {
    pub fn new(frequency: c_int) -> Timer {
        let interval = 1e6 as i64 / i64::from(frequency);
        let it_interval = Timeval {
            tv_sec: interval / 1e6 as i64,
            tv_usec: interval % 1e6 as i64,
        };

        let (cx, rx) = mpsc::channel();
        let (cx_cancel, mut rx_cancel) = mpsc::channel();
        std::thread::spawn(move || {
            while rx_cancel.try_recv().is_err() {
                std::thread::sleep(Duration::from_micros(it_interval.tv_usec as u64));
                perf_signal_handler();
                // cx.send(()).unwrap();
            }
        });

        Timer {
            _frequency: frequency,
            receiver: rx,
            _cancel_sender: Some(cx_cancel),
        }
    }
}

impl std::ops::Drop for Timer {
    fn drop(&mut self) {
        if let Some(sender) = self._cancel_sender.take() {
            sender.send(());
        }
    }
}
