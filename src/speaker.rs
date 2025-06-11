use core::{
    cell::RefCell,
    option::Option::{self, None, Some},
};
use cortex_m::interrupt::{Mutex, free};
use embedded_hal::digital::OutputPin;
use microbit::{
    hal::{
        gpio::{
            Disconnected,
            Level,
            p0::P0_00,
        },
        pwm,
        rtc,
        time::Hertz,
    },
    pac::{self, interrupt},
};

const INITIAL_FREQUENCY : u32 = 500;

static SPEAKER: Mutex<RefCell<Option<Speaker>>> = Mutex::new(RefCell::new(None));

pub struct Speaker {
    rtc: rtc::Rtc<pac::RTC0>,
    pwm: pwm::Pwm<pac::PWM0>,
    fq: u32,
    is_on: bool,
}

impl Speaker {
    /// initialize the speaker
    pub fn init(speaker_pin: P0_00<Disconnected>, pwm_per: pac::PWM0, rtc_per: pac::RTC0, mut nvic: pac::NVIC) {
        free(move |cs| {
            let mut rtc_unit = rtc::Rtc::new(rtc_per, 500).unwrap();
            rtc_unit.enable_counter();
            rtc_unit.enable_interrupt(rtc::RtcInterrupt::Tick, Some(&mut nvic));
            
            let mut speaker_pin = speaker_pin.into_push_pull_output(Level::High);
            let _ = speaker_pin.set_low();

            let pwm_unit = pwm::Pwm::new(pwm_per);
            pwm_unit
                .set_output_pin(pwm::Channel::C0, speaker_pin.degrade())
                .set_prescaler(pwm::Prescaler::Div8)
                .set_period(Hertz(INITIAL_FREQUENCY))
                .set_counter_mode(pwm::CounterMode::UpAndDown)
                .set_max_duty(5000)
                .enable();

            let max_duty = pwm_unit.max_duty();
            pwm_unit.set_duty_on_common(max_duty / 2);

            *SPEAKER.borrow(cs).borrow_mut() = Some(Self {
                rtc: rtc_unit,
                pwm: pwm_unit,
                fq: INITIAL_FREQUENCY,
                is_on: true,
            });

            unsafe {
                pac::NVIC::unmask(pac::Interrupt::RTC0);
                pac::NVIC::unpend(pac::Interrupt::RTC0);
            }

        });
    }

    pub fn toggle() {
        free(|cs| {
            if let Some(speaker) = SPEAKER.borrow(cs).borrow_mut().as_mut() {
                match speaker.is_on {
                    true => {
                        speaker.pwm.stop();
                    },
                    false => {
                        speaker.pwm
                            .set_period(Hertz(speaker.fq))
                            .set_max_duty(5000)
                            .enable();

                        let max_duty = speaker.pwm.max_duty();
                        speaker.pwm.set_duty_on_common(max_duty / 2);
                    },
                }
                speaker.is_on = !speaker.is_on;
            }
        });
    }

    pub fn stop() {
        free(|cs| {
            if let Some(speaker) = SPEAKER.borrow(cs).borrow_mut().as_mut() {
                speaker.pwm.stop();
                speaker.is_on = false;
            }
        });
    }

    pub fn start() {
        free(|cs| {
            if let Some(speaker) = SPEAKER.borrow(cs).borrow_mut().as_mut() {
                speaker.pwm
                    .set_period(Hertz(speaker.fq))
                    .set_max_duty(5000)
                    .enable();

                let max_duty = speaker.pwm.max_duty();
                speaker.pwm.set_duty_on_common(max_duty / 2);
                speaker.is_on = true;
            }
        });
    }

    pub fn play(freq: u32) {
        free(|cs| {
            if let Some(speaker) = SPEAKER.borrow(cs).borrow_mut().as_mut() {
                speaker.pwm.set_period(Hertz(freq));
                let max_duty = speaker.pwm.max_duty();
                speaker.pwm.set_duty_on_common(max_duty / 2);
            }
        });
    }
}

#[interrupt]
fn RTC0() {
    free(|cs| {
        if let Some(speaker) = SPEAKER.borrow(cs).borrow().as_ref() {
            speaker.rtc.reset_event(rtc::RtcInterrupt::Tick);
        }
    });
}
