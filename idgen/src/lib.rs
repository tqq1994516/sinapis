pub mod core;

pub use core::*;

pub fn set_id_generator(options: IdGeneratorOptions) {
    IdHelper::set_id_generator(options);
}

pub fn set_options(worker_id: u32, worker_id_bit_length: u8, seq_bit_length: u8) {
    let mut options = IdGeneratorOptions::new(worker_id);
    options.worker_id_bit_length = worker_id_bit_length;
    options.seq_bit_length = seq_bit_length;
    IdHelper::set_id_generator(options);
}

pub fn set_worker_id(worker_id: u32) {
    IdHelper::set_worker_id(worker_id);
}

pub fn next_id() -> i64 {
    IdHelper::next_id()
}
