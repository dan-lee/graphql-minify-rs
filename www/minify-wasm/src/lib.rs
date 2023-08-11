use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;

mod utils;

cfg_if! {
    if #[cfg(feature = "wee_alloc")] {
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
pub fn minify(query: &str) -> String {
  graphql_minify::minify(query).unwrap()
}
