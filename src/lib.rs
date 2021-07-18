extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

extern crate unicode_segmentation;
extern crate unicode_width;

mod parinfer;
mod types;
mod changes;

#[macro_use]
#[cfg(feature = "emacs")]
extern crate emacs;
// Native-specific stuff

#[cfg(not(target_arch = "wasm32"))]
extern crate libc;

#[cfg(not(target_arch = "wasm32"))]
mod c_wrapper;

#[cfg(not(target_arch = "wasm32"))]
pub use c_wrapper::run_parinfer;

#[cfg(not(target_arch = "wasm32"))]
pub use c_wrapper::INITIALIZED;

#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "emacs")]
mod emacs_wrapper;

#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "emacs")]
pub use emacs_wrapper::init;

// WebAssembly-specific stuff

#[cfg(target_arch = "wasm32")]
#[macro_use]
extern crate stdweb;

#[cfg(target_arch = "wasm32")]
use stdweb::js_export;

mod common_wrapper;

#[cfg(target_arch = "wasm32")]
mod wasm_wrapper;

#[cfg(target_arch = "wasm32")]
#[js_export]
pub fn run_parinfer(input: String) -> String {
    wasm_wrapper::run_parinfer(input)
}

#[cfg(windows)]
extern crate winapi;
