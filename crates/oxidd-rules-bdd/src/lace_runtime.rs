#[cfg(feature = "lace")]
use std::sync::{Mutex, OnceLock};

#[cfg(feature = "lace")]
static LACE: OnceLock<Mutex<lace::Lace>> = OnceLock::new();

#[cfg(feature = "lace")]
pub(crate) fn with_lace<R>(threads: usize, f: impl FnOnce(&mut lace::Lace) -> R) -> R {
    let lace = LACE.get_or_init(|| {
        Mutex::new(lace::Lace::init(threads.max(1)))
    });

    let mut lace = lace.lock().expect("Lace runtime mutex poisoned");
    f(&mut lace)
}