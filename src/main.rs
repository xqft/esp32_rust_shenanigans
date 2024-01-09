#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

extern crate alloc;

use core::mem::MaybeUninit;
use embassy_executor::Spawner;
use embassy_time::Timer;
use esp32_shenanigans::motor::{self, SingleMotorConfig};
use esp_backtrace as _;
use hal::{
    clock::ClockControl,
    gpio::{AnyPin, Output, PushPull},
    mcpwm::{
        operator::{PwmActions, PwmPinConfig, PwmUpdateMethod},
        timer::PwmWorkingMode,
        PeripheralClockConfig, MCPWM,
    },
    peripherals::Peripherals,
    prelude::*,
    IO,
};
use hal::{
    embassy::{self},
    timer::TimerGroup,
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

//async fn handle_motor(mut motor_pin: )

#[main]
async fn main(_spawner: Spawner) {
    init_heap();
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // setup timer0
    let clocks = ClockControl::max(system.clock_control).freeze();
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timer_group0.timer0);

    // motor pwm
    let mut motor_config = SingleMotorConfig::take(
        io.pins.gpio32.into_push_pull_output(),
        peripherals.MCPWM0,
        &clocks,
    );

    for i in 0..5 {
        motor_config.set_duty_cycle(1.0 / 5.0 * i as f32);
        Timer::after_secs(1).await;
    }
    for i in (0..5).rev() {
        motor_config.set_duty_cycle(1.0 / 5.0 * i as f32);
        Timer::after_secs(1).await;
    }

    // setup logger
    esp_println::logger::init_logger_from_env();
    log::info!("Logger is setup");

    let led_pin = io.pins.gpio2.into_push_pull_output().into();
    handle_led(led_pin).await;
}
