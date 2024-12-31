use std::sync::OnceLock;

use super::DefaultIdGenerator;
use super::IdGeneratorOptions;

pub struct IdHelper;

static ID_GEN_INSTANCE: OnceLock<DefaultIdGenerator> = OnceLock::new();

impl IdHelper {
    fn id_gen_instance() -> DefaultIdGenerator {
        *ID_GEN_INSTANCE.get_or_init(|| DefaultIdGenerator::default())
    }

    pub fn set_id_generator(options: IdGeneratorOptions) {
        let mut idgen = IdHelper::id_gen_instance();
        idgen.worker.set_options(options);
    }

    pub fn set_worker_id(worker_id: u32) {
        let mut idgen = IdHelper::id_gen_instance();
        let options = IdGeneratorOptions::new(worker_id);
        idgen.worker.set_options(options);
    }

    pub fn next_id() -> i64 {
        let mut idgen = IdHelper::id_gen_instance();
        idgen.worker.next_id()
    }
}
