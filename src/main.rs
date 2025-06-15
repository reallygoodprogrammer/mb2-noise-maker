#![no_main]
#![no_std]
use cortex_m_rt::entry;
use embedded_hal::digital::InputPin;
use microbit::Board;
use panic_halt as _;

use noiser::{display::Display, notes::Notes, rand::Rand, speaker::Speaker};

#[entry]
fn main() -> ! {
    let board = Board::take().unwrap();
    let mut buttons = board.buttons;

    Rand::init(board.RNG);
    Display::init(board.TIMER1, board.TIMER2, board.display_pins);
    Speaker::init(board.speaker_pin, board.PWM0, board.RTC0, board.NVIC);
    Speaker::stop();

    let mut pressed = false;
    loop {
        match (
            buttons.button_a.is_low().unwrap(),
            buttons.button_b.is_low().unwrap(),
            pressed,
        ) {
            (true, false, false) => {
                Display::running();
                pressed = true;
            }
            (true, true, true) => {
                Display::running();
                pressed = true;
            }
            (false, true, true) => {
                Display::running();
                pressed = true;
            }
            (false, false, true) => {
                Display::idle();
                pressed = false;
            }
            _ => (),
        }
    }
}
