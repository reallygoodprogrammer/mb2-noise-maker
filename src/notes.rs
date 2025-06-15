use crate::{rand::RAND, speaker::Speaker};
use core::{
    cell::RefCell,
    option::Option::{self, None, Some},
};
use cortex_m::interrupt::{Mutex, free};
use microbit::{
    hal::Timer,
    pac::{self, TIMER3, interrupt},
};

pub enum NoteMode {
    LFO,
    RandomNote,
}

static NOTES: Mutex<RefCell<Option<Notes>>> = Mutex::new(RefCell::new(None));

pub struct Notes {
    mode: NoteMode,
    freq: u32,
    delay: u32,
    timer: Timer<TIMER3>,
    enabled: bool,
}

impl Notes {
    pub fn init(timer: TIMER3) {
        free(|cs| {
            let timer = Timer::new(timer);
            unsafe {
                pac::NVIC::unmask(pac::Interrupt::TIMER3);
            }
            *NOTES.borrow(cs).borrow_mut() = Some(Self {
                mode: NoteMode::RandomNote,
                freq: 500u32,
                delay: 200000u32,
                timer,
                enabled: false,
            });
        });
    }

    pub fn update() -> Option<u32> {
        free(|cs| {
            if let Some(notes) = NOTES.borrow(cs).borrow_mut().as_mut() {
                if notes.enabled {
                    match notes.mode {
                        NoteMode::LFO => (),
                        NoteMode::RandomNote => {
                            if let Some(rng) = RAND.borrow(cs).borrow_mut().as_mut() {
                                notes.freq = rng.rand_u32(300, 1400);
                            }
                        }
                    }
                    notes.timer.reset_event();
                    notes.timer.start(notes.delay);
                    Some(notes.freq)
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    pub fn get_note() -> Option<u32> {
        free(|cs| {
            if let Some(notes) = NOTES.borrow(cs).borrow_mut().as_mut() {
                Some(notes.freq)
            } else {
                None
            }
        })
    }

    pub fn mode(new_mode: NoteMode) {
        free(|cs| {
            if let Some(notes) = NOTES.borrow(cs).borrow_mut().as_mut() {
                notes.mode = new_mode;
            }
        });
    }

    pub fn enable() {
        free(|cs| {
            if let Some(notes) = NOTES.borrow(cs).borrow_mut().as_mut() {
                notes.enabled = true;
                notes.timer.enable_interrupt();
                notes.timer.start(notes.delay);
            }
        });
    }

    pub fn disable() {
        free(|cs| {
            if let Some(notes) = NOTES.borrow(cs).borrow_mut().as_mut() {
                notes.enabled = false;
                notes.timer.disable_interrupt();
            }
        });
    }

    pub fn toggle() {
        free(|cs| {
            if let Some(notes) = NOTES.borrow(cs).borrow_mut().as_mut() {
                match notes.mode {
                    NoteMode::LFO => (),
                    NoteMode::RandomNote => {
                        if let Some(rng) = RAND.borrow(cs).borrow_mut().as_mut() {
                            notes.delay = rng.rand_u32(25000, 100000);
                        }
                    }
                }
            }
        });
    }
}

#[interrupt]
fn TIMER3() {
    Speaker::play(Notes::update().unwrap_or(500u32));
}
