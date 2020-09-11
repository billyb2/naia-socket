cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        // Wasm //
        use js_sys::Math::random;
        pub fn gen_range_f32(lower: f32, upper: f32) -> f32 {
            let rand_range: f32 = random() as f32 * (upper - lower);
            return rand_range + lower;
        }
        pub fn gen_range_u32(lower: u32, upper: u32) -> u32 {
            let rand_range: u32 = (random() * f64::from(upper - lower)) as u32;
            return rand_range + lower;
        }
        pub fn gen_bool() -> bool {
            return random() < 0.5;
        }

    } else {
        // Linux //
        use rand::Rng;
        pub fn gen_range_f32(lower: f32, upper: f32) -> f32 {
            return rand::thread_rng().gen_range(lower, upper);
        }
        pub fn gen_range_u32(lower: u32, upper: u32) -> u32 {
            return rand::thread_rng().gen_range(lower, upper);
        }
        pub fn gen_bool() -> bool {
            return rand::thread_rng().gen_bool(0.5);
        }
    }
}
