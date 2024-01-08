#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

extern crate alloc;

use core::mem::MaybeUninit;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_println::println;
use hal::{
    clock::ClockControl,
    embassy::executor::Executor,
    gpio::{AnyPin, Output, PushPull},
    peripherals::Peripherals,
    prelude::*,
    Delay, IO,
};

use hal::{
    embassy::{self, executor},
    timer::TimerGroup,
    Rng,
};
#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    const HEAP_SIZE: usize = 32 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE);
    }
}

async fn handle_led(mut led_pin: AnyPin<Output<PushPull>>) {
    loop {
        led_pin.toggle().unwrap();
        Timer::after_secs(1).await;
    }
}

#[main]
async fn main(_spawner: Spawner) {
    init_heap();
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let clocks = ClockControl::max(system.clock_control).freeze();

    // setup logger
    esp_println::logger::init_logger_from_env();
    log::info!("Logger is setup");

    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timer_group0.timer0);

    let led_pin = io.pins.gpio2.into_push_pull_output().into();

    handle_led(led_pin).await;
}
