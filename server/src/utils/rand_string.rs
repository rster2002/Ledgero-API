use rand::{thread_rng, Rng};

const CHARSET: &[u8] = b"0123456789abcdef";

pub fn rand_string(length: u32) -> String {
    let mut rng = thread_rng();

    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
