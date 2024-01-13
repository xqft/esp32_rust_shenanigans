#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

extern crate alloc;

use core::mem::MaybeUninit;
use embassy_executor::Spawner;
use embassy_time::Timer;
use esp32_shenanigans::motor::{self, DoubleMotorConfig, SingleMotorConfig};
use esp_backtrace as _;
use esp_println::println;
use hal::{
    adc::{AdcConfig, Attenuation, ADC},
    analog::{ADC1, ADC2},
    clock::ClockControl,
    delay,
    gpio::{AnyPin, Output, PushPull},
    mcpwm::{
        operator::{PwmActions, PwmPinConfig, PwmUpdateMethod},
        timer::PwmWorkingMode,
        PeripheralClockConfig, MCPWM,
    },
    peripherals::Peripherals,
    prelude::*,
    Delay, IO,
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
    let mut motor_config = DoubleMotorConfig::take(
        io.pins.gpio32.into_push_pull_output(),
        io.pins.gpio33.into_push_pull_output(),
        peripherals.MCPWM0,
        &clocks,
    );

    // setup potentiometer on pin 35 for motor speed control
    let analog = peripherals.SENS.split();
    let mut adc1_config = AdcConfig::new();
    let mut pin35 =
        adc1_config.enable_pin(io.pins.gpio35.into_analog(), Attenuation::Attenuation11dB);
    let mut adc1 = ADC::<ADC1>::adc(analog.adc1, adc1_config).unwrap();

    // setup logger
    esp_println::logger::init_logger_from_env();
    log::info!("Logger is setup");

    let led_pin = io.pins.gpio2.into_push_pull_output().into();
    handle_led(led_pin);

    loop {
        let pot_val = (nb::block!(adc1.read(&mut pin35)).unwrap() as u16) as f32 / 4000 as f32;
        motor_config.set_duty_cycle_a(pot_val);
        motor_config.set_duty_cycle_b(pot_val);
        println!("val: {}", nb::block!(adc1.read(&mut pin35)).unwrap() as u16);
        println!("val_normal: {}", pot_val);
        Timer::after_millis(200).await;
    }
}
