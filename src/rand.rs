use core::{
    cell::RefCell,
    option::Option::{self, None, Some},
};
use cortex_m::interrupt::{Mutex, free};
use microbit::hal::{Rng as MBRng, pac::RNG};
use nanorand::{Pcg64, Rng, SeedableRng};

pub static RAND: Mutex<RefCell<Option<Rand>>> = Mutex::new(RefCell::new(None));

pub struct Rand {
    rng: Pcg64,
}

impl Rand {
    pub fn init(mbrng_perph: RNG) {
        free(|cs| {
            let mut rng = nanorand::Pcg64::new_seed(1);
            let mut seed = [0u8; 16];
            let mut mbrng = MBRng::new(mbrng_perph);
            mbrng.random(&mut seed);
            rng.reseed(seed);
            *RAND.borrow(cs).borrow_mut() = Some(Self { rng });
        });
    }

    pub fn rand_u8(&mut self, low: u8, high: u8) -> u8 {
        (self.rng.generate::<u8>() + low) % high
    }

    pub fn rand_u32(&mut self, low: u32, high: u32) -> u32 {
        (self.rng.generate::<u32>() + low) % high
    }
}
