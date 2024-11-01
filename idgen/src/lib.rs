pub mod core;

pub use core::*;

pub fn SetIdGenerator(options: IdGeneratorOptions) {
    YitIdHelper::SetIdGenerator(options);
}

pub fn SetOptions(workerId: u32, workerIdBitLength: u8, seqBitLength: u8) {
    let mut options = IdGeneratorOptions::New(1);
    options.WorkerIdBitLength = workerIdBitLength;
    options.SeqBitLength = seqBitLength;
    YitIdHelper::SetIdGenerator(options);
}

pub fn SetWorkerId(workerId: u32) {
    YitIdHelper::SetWorkerId(workerId);
}

pub fn NextId() -> i64 {
    YitIdHelper::NextId()
}

// build-win-x64: cargo build --release
// build-linux-x64: cargo build --target x86_64-unknown-linux-musl --release
