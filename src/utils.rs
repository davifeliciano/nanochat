use rand::Rng;

pub fn compute_random_32_bytes_key() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill(&mut bytes);
    hex::encode(bytes)
}
