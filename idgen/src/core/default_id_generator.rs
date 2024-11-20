/*
 * 版权属于：yitter(yitter@126.com)
 * 开源地址：https://github.com/yitter/idgenerator
 */
use std::{thread, time};
use std::net::UdpSocket;
use chrono::Utc;
use std::sync::Mutex;
use std::sync::Arc;
use std::borrow::BorrowMut;

use super::snow_worker::SnowWorker;

// static mut instance2: Option<Arc<Mutex<SnowWorkerM1>>> = None;

pub struct DefaultIdGenerator {
    pub Worker: SnowWorker,
}

impl DefaultIdGenerator {
    pub fn Default() -> DefaultIdGenerator {
        DefaultIdGenerator { Worker: SnowWorker::Default() }
    }
}
