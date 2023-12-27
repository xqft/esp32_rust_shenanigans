#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

extern crate alloc;
extern crate embassy_esp32;

use embassy_esp32::peripherals::ultrasonic::Ultrasonic;
use embassy_executor::{Executor, _export::StaticCell};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_println::println;
use hal::{
    clock::ClockControl,
    embassy,
    gpio::{AnyPin, Output, PushPull},
    mcpwm::{operator::PwmPinConfig, timer::PwmWorkingMode, PeripheralClockConfig, MCPWM},
    peripherals::Peripherals,
    prelude::*,
    timer::TimerGroup,
    Rtc, IO,
};

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

fn init_heap() {
    const HEAP_SIZE: usize = 32 * 1024;

    extern "C" {
        static mut _heap_start: u32;
        static mut _heap_end: u32;
    }

    unsafe {
        let heap_start = &_heap_start as *const _ as usize;
        let heap_end = &_heap_end as *const _ as usize;
        assert!(
            heap_end - heap_start > HEAP_SIZE,
            "Not enough available heap memory."
        );
        ALLOCATOR.init(heap_start as *mut u8, HEAP_SIZE);
    }
}

#[embassy_executor::task]
async fn handle_ultrasonic1(mut ultrasonic: Ultrasonic) {
    loop {
        let distance = ultrasonic.distance().await;
        println!("{:.0} mm", distance,);
        Timer::after(Duration::from_millis(10)).await;
    }
}

#[embassy_executor::task]
async fn handle_ultrasonic2(mut ultrasonic: Ultrasonic) {
    loop {
        let distance = ultrasonic.distance().await;
        println!("{:.0} mm", distance,);
        Timer::after(Duration::from_millis(10)).await;
    }
}

#[embassy_executor::task]
async fn handle_led(mut led_pin: AnyPin<Output<PushPull>>) {
    loop {
        led_pin.toggle().unwrap();
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[entry]
fn main() -> ! {
    init_heap();
    let peripherals = Peripherals::take();
    let mut system = peripherals.DPORT.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(
        peripherals.TIMG1,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt1 = timer_group1.wdt;
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    // Init timer 0
    //#[cfg(feature = "embassy-time-timg0")]
    embassy::init(&clocks, timer_group0.timer0);

    let clock_cfg = PeripheralClockConfig::with_frequency(&clocks, 40u32.MHz()).unwrap();
    let mut mcpwm = MCPWM::new(
        peripherals.MCPWM0,
        clock_cfg,
        &mut system.peripheral_clock_control,
    );

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let led_pin = io.pins.gpio2.into_push_pull_output().into();

    mcpwm.operator0.set_timer(&mcpwm.timer0);
    let mut pwm_pin = mcpwm.operator0.with_pin_a(
        io.pins.gpio32.into_push_pull_output(),
        PwmPinConfig::UP_ACTIVE_HIGH,
    );

    let timer_clock_cfg = clock_cfg
        .timer_clock_with_frequency(99, PwmWorkingMode::Increase, 5u32.kHz())
        .unwrap();
    mcpwm.timer0.start(timer_clock_cfg);

    pwm_pin.set_timestamp(50);

    // let ultrasonic1 = {
    //     let trig_pin = io.pins.gpio19.into_push_pull_output().into();
    //     let echo_pin = io.pins.gpio18.into_pull_down_input().into();
    //     Ultrasonic::new(trig_pin, echo_pin)
    // };
    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        //spawner.spawn(handle_ultrasonic1(ultrasonic1)).ok();
        //spawner.spawn(handle_ultrasonic2(ultrasonic2)).ok();
        spawner.spawn(handle_led(led_pin)).ok();
    });
}
