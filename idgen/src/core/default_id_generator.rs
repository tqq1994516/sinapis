use super::snow_worker::SnowWorker;

// static mut instance2: Option<Arc<Mutex<SnowWorkerM1>>> = None;

#[derive(Copy, Clone)]
pub struct DefaultIdGenerator {
    pub worker: SnowWorker,
}

impl DefaultIdGenerator {
    pub fn default() -> Self {
        Self { worker: SnowWorker::default() }
    }
}
