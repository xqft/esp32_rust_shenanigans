use core::{
    ops::RangeInclusive,
    sync::atomic::{AtomicBool, Ordering},
};

use hal::{
    clock::Clocks,
    gpio::OutputPin,
    mcpwm::{
        operator::{PwmActions, PwmPin, PwmPinConfig, PwmUpdateMethod},
        timer::PwmWorkingMode,
        PeripheralClockConfig, MCPWM,
    },
    peripheral::Peripheral,
    peripherals::{Peripherals, MCPWM0},
    prelude::_fugit_RateExtU32,
};

/// We define MotorPWMPinA for using MCPWM0, operator 0 and pin A.
type MotorPWMPinA<'d, Pin: OutputPin> = PwmPin<'d, Pin, MCPWM0, 0, true>;
/// We define MotorPWMPinB for using MCPWM0, operator 0 and pin B.
type MotorPWMPinB<'d, Pin: OutputPin> = PwmPin<'d, Pin, MCPWM0, 0, false>;

/// True if a single motor configuration has been initialized.
static _SINGLE_MOTOR_CONFIG_INIT: AtomicBool = AtomicBool::new(false);
/// True if a double motor configuration has been initialized.
static _DOUBLE_MOTOR_CONFIG_INIT: AtomicBool = AtomicBool::new(false);

/// A singleton structure that defines a single motor configuration.
pub struct SingleMotorConfig<'d, Pin>
where
    Pin: OutputPin + 'd,
{
    pwm_pin: MotorPWMPinA<'d, Pin>,
}

impl<'d, Pin: OutputPin> SingleMotorConfig<'d, Pin> {
    const PERIOD: u16 = 256;

    pub fn take(pin: impl Peripheral<P = Pin> + 'd, mcpwm0: MCPWM0, clocks: &Clocks) -> Self {
        if _SINGLE_MOTOR_CONFIG_INIT.load(Ordering::Relaxed) {
            panic!("single motor config initialized more than once!");
        }
        if _DOUBLE_MOTOR_CONFIG_INIT.load(Ordering::Relaxed) {
            panic!("double motor config initialized more than once!");
        }
        _SINGLE_MOTOR_CONFIG_INIT.store(true, Ordering::Relaxed);

        // configure clock
        let clock_cfg = PeripheralClockConfig::with_frequency(clocks, 40u32.MHz()).unwrap();
        let mut mcpwm = MCPWM::new(mcpwm0, clock_cfg);
        mcpwm.operator0.set_timer(&mcpwm.timer0);
        let pwm_pin = mcpwm.operator0.with_pin_a(
            pin,
            PwmPinConfig::new(PwmActions::UP_ACTIVE_HIGH, PwmUpdateMethod::SYNC_IMMEDIATLY),
        );

        let timer_clock_cfg = clock_cfg
            .timer_clock_with_frequency(Self::PERIOD, PwmWorkingMode::Increase, 10u32.kHz())
            .unwrap();
        mcpwm.timer0.start(timer_clock_cfg);

        SingleMotorConfig { pwm_pin }
    }

    pub fn set_duty_cycle(&mut self, duty_cycle: f32) {
        if !RangeInclusive::new(0., 1.).contains(&duty_cycle) {
            panic!("duty cycle must be between 0 and 1 inclusive.");
        }

        self.pwm_pin
            .set_timestamp((Self::PERIOD as f32 * duty_cycle) as u16);
    }
}

/// A singleton structure that defines a pair of motors configuration.
pub struct DoubleMotorConfig<'d, PinA, PinB>
where
    PinA: OutputPin + 'd,
    PinB: OutputPin + 'd,
{
    pwm_pin_a: MotorPWMPinA<'d, PinA>,
    pwm_pin_b: MotorPWMPinB<'d, PinB>,
}

impl<'d, PinA: OutputPin, PinB: OutputPin> DoubleMotorConfig<'d, PinA, PinB> {
    const PERIOD: u16 = 256;

    pub fn take(
        pin_a: impl Peripheral<P = PinA> + 'd,
        pin_b: impl Peripheral<P = PinB> + 'd,
        mcpwm0: MCPWM0,
        clocks: &Clocks,
    ) -> Self {
        if _SINGLE_MOTOR_CONFIG_INIT.load(Ordering::Relaxed) {
            panic!("single motor config initialized more than once!");
        }
        if _DOUBLE_MOTOR_CONFIG_INIT.load(Ordering::Relaxed) {
            panic!("double motor config initialized more than once!");
        }
        _DOUBLE_MOTOR_CONFIG_INIT.store(true, Ordering::Relaxed);

        // configure clock
        let clock_cfg = PeripheralClockConfig::with_frequency(clocks, 40u32.MHz()).unwrap();
        let mut mcpwm = MCPWM::new(mcpwm0, clock_cfg);
        mcpwm.operator0.set_timer(&mcpwm.timer0);
        let (pwm_pin_a, pwm_pin_b) = mcpwm.operator0.with_pins(
            pin_a,
            PwmPinConfig::new(PwmActions::UP_ACTIVE_HIGH, PwmUpdateMethod::SYNC_IMMEDIATLY),
            pin_b,
            PwmPinConfig::new(PwmActions::UP_ACTIVE_HIGH, PwmUpdateMethod::SYNC_IMMEDIATLY),
        );

        let timer_clock_cfg = clock_cfg
            .timer_clock_with_frequency(Self::PERIOD, PwmWorkingMode::Increase, 10u32.kHz())
            .unwrap();
        mcpwm.timer0.start(timer_clock_cfg);

        DoubleMotorConfig {
            pwm_pin_a,
            pwm_pin_b,
        }
    }

    pub fn set_duty_cycle_a(&mut self, duty_cycle: f32) {
        if !RangeInclusive::new(0., 1.).contains(&duty_cycle) {
            panic!("duty cycle must be between 0 and 1 inclusive.");
        }

        self.pwm_pin_a
            .set_timestamp((Self::PERIOD as f32 * duty_cycle) as u16);
    }

    pub fn set_duty_cycle_b(&mut self, duty_cycle: f32) {
        if !RangeInclusive::new(0., 1.).contains(&duty_cycle) {
            panic!("duty cycle must be between 0 and 1 inclusive.");
        }

        self.pwm_pin_b
            .set_timestamp((Self::PERIOD as f32 * duty_cycle) as u16);
    }
}
