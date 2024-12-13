use super::snow_worker::SnowWorker;

// static mut instance2: Option<Arc<Mutex<SnowWorkerM1>>> = None;

pub struct DefaultIdGenerator {
    pub worker: SnowWorker,
}

impl DefaultIdGenerator {
    pub fn default() -> Self {
        Self { worker: SnowWorker::default() }
    }
}
