use core::{
    cell::RefCell,
    iter::Iterator,
    option::Option::{self, None, Some},
};
use cortex_m::interrupt::{Mutex, free};
use microbit::{
    display::nonblocking,
    gpio::DisplayPins,
    hal::Timer,
    pac::{self, TIMER1, TIMER2, interrupt},
};

use crate::rand::RAND;

const BLANK: [[u8; 5]; 5] = [
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
];

const IDLE_DELAY: u32 = 140000;
const RUNNING_DELAY: u32 = 80000;

static DISPLAY: Mutex<RefCell<Option<Display>>> = Mutex::new(RefCell::new(None));

/// Mode
///
/// Corresponds to the state of the noise maker i.e. running and making sound,
/// or in an idle state.
enum Mode {
    Running,
    Idle,
}

/// Display
///
/// Display struct for encompassing all the different parts necessary to handle
/// the display functionality
pub struct Display {
    display: nonblocking::Display<TIMER1>,
    timer: Timer<TIMER2>,
    position: [(u8, u8); 5],
    max: usize,
    delay: u32,
    mode: Mode,
}

impl Display {
    /// Init
    ///
    /// Initialize and start the display.
    pub fn init(dtimer: TIMER1, ctimer: TIMER2, pins: DisplayPins) {
        let mut display = nonblocking::Display::new(dtimer, pins);
        let mut timer = Timer::new(ctimer);
        let delay = IDLE_DELAY;
        display.show(&nonblocking::GreyscaleImage::new(&[
            [9, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
        ]));
        free(move |cs| {
            unsafe {
                pac::NVIC::unmask(pac::Interrupt::TIMER1);
                pac::NVIC::unmask(pac::Interrupt::TIMER2);
            }
            timer.enable_interrupt();
            timer.start(delay);
            *DISPLAY.borrow(cs).borrow_mut() = Some(Self {
                display,
                timer,
                position: [(0, 0); 5],
                max: 4,
                delay,
                mode: Mode::Idle,
            });
        });
    }

    /// Start displaying the next frame of the animation.
    pub fn next() {
        let mut next_frame = BLANK;
        free(|cs| {
            if let Some(d) = DISPLAY.borrow(cs).borrow_mut().as_mut() {
                match d.mode {
                    Mode::Idle => {
                        for i in 0..d.max {
                            d.position[i] = d.position[i + 1];
                        }
                        d.position[d.max] = match (d.position[d.max].0, d.position[d.max].1) {
                            (4, 4) => (0, 0),
                            (4, _) => (0, d.position[d.max].1 + 1),
                            _ => (d.position[d.max].0 + 1, d.position[d.max].1),
                        };

                        for (i, p) in d.position.iter().enumerate() {
                            next_frame[p.0 as usize][p.1 as usize] = match i {
                                0 => 1,
                                1 => 3,
                                2 => 5,
                                3 => 7,
                                4 => 9,
                                _ => 0,
                            };
                        }
                        d.display
                            .show(&nonblocking::GreyscaleImage::new(&next_frame));
                    }
                    Mode::Running => {
                        if let Some(rng) = RAND.borrow(cs).borrow_mut().as_mut() {
                            for i in 0..d.max {
                                d.position[i] = d.position[i + 1];
                            }
                            let rand_val = rng.rand_u8(0, 24);
                            let x = rand_val % 5;
                            let y = (rand_val - x) / 5;
                            d.position[d.max] = (x, y);
                        }

                        for (i, p) in d.position.iter().enumerate() {
                            next_frame[p.0 as usize][p.1 as usize] = match i {
                                0 => 1,
                                1 => 3,
                                2 => 5,
                                3 => 7,
                                4 => 9,
                                _ => 0,
                            };
                        }
                        d.display
                            .show(&nonblocking::GreyscaleImage::new(&next_frame));
                    }
                }

                d.timer.reset_event();
                d.timer.start(d.delay);
            }
        });
    }

    /// Set the display into its idle mode.
    pub fn idle() {
        free(|cs| {
            if let Some(d) = DISPLAY.borrow(cs).borrow_mut().as_mut() {
                d.mode = Mode::Idle;
                d.delay = IDLE_DELAY;
            }
        })
    }

    /// Set the display into its running mode (default).
    pub fn running() {
        free(|cs| {
            if let Some(d) = DISPLAY.borrow(cs).borrow_mut().as_mut() {
                d.mode = Mode::Running;
                d.delay = RUNNING_DELAY;
            }
        })
    }
}

#[interrupt]
fn TIMER1() {
    free(|cs| {
        if let Some(d) = DISPLAY.borrow(cs).borrow_mut().as_mut() {
            d.display.handle_display_event();
        }
    });
}

#[interrupt]
fn TIMER2() {
    Display::next();
}
