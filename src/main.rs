#![no_main]
#![no_std]
use cortex_m_rt::entry;
use embedded_hal::digital::InputPin;
use microbit::Board;
use panic_halt as _;

use noiser::{
    display::Display,
    notes::{NoteMode, Notes},
    rand::Rand,
    speaker::Speaker,
};

#[entry]
fn main() -> ! {
    let board = Board::take().unwrap();
    let mut buttons = board.buttons;

    Rand::init(board.RNG);
    Display::init(board.TIMER1, board.TIMER2, board.display_pins);
    Notes::init(board.TIMER3);
    Speaker::init(board.speaker_pin, board.PWM0, board.RTC0, board.NVIC);
    Speaker::stop();

    let mut pressed = false;
    loop {
        match (
            buttons.button_a.is_low().unwrap(),
            buttons.button_b.is_low().unwrap(),
            pressed,
        ) {
            (false, false, false) => (),
            (false, false, true) => {
                Display::idle();
                Notes::disable();
                Speaker::stop();
                pressed = false;
            }
            (a, b, p) => {
                if !p {
                    Display::running();
                    Speaker::start();
                    Notes::enable();
                    pressed = true;
                    if a {
                        Notes::mode(NoteMode::RandomNote);
                    } else {
                        Notes::mode(NoteMode::LFO);
                    }
                }

                if a && b {
                    Notes::toggle();
                }
            }
        }
    }
}
