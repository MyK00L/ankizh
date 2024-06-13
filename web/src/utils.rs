macro_rules! add_callback {
    ($($ev:literal),*, $et:ident, $($tc:ident),*, $(||)? $(|)? $($i:ident: $t:ty),* $(|)? $cl: block) => {
        macro_rules! inner{
            ($iev: literal) => {
                {
                $(
                    let $tc = $tc.clone();
                )*
                let fun = Closure::<dyn Fn($($t)*)>::new(move | $($i: $t),* | $cl);
                $et.add_event_listener_with_callback($iev, fun.as_ref().unchecked_ref()).unwrap();
                fun.forget();
                }

            }
        }
        $(
            inner!($ev);
        )*
    };
}
pub(crate) use add_callback;

use wasm_bindgen::prelude::wasm_bindgen;
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}
macro_rules! console_log {
    ($($t:tt)*) => (crate::utils::log(&format_args!($($t)*).to_string()))
}
pub(crate) use console_log;
