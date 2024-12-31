use chrono::Utc;
use std::thread::sleep;

use super::{IdGeneratorOptions, OverCostActionArg};

#[derive(Copy, Clone)]
pub struct SnowWorker {
    ///基础时间
    pub base_time: i64,
    ///机器码
    pub worker_id: u32,
    ///机器码位长
    pub worker_id_bit_length: u8,
    ///自增序列数位长
    pub seq_bit_length: u8,
    ///最大序列数（含）
    pub max_seq_number: u32,
    ///最小序列数（含）
    pub min_seq_number: u32,
    ///最大漂移次数
    pub top_over_cost_count: u32,

    timestamp_shift: u8,
    current_seq_number: u32,
    last_time_tick: i64,
    turn_back_time_tick: i64,
    turn_back_index: u8,
    is_over_cost: bool,
    over_cost_count_in_one_term: u32,
    gen_count_in_one_term: u32,
    term_index: u32,
}

impl SnowWorker {
    pub fn default() -> SnowWorker {
        let options = IdGeneratorOptions::new(1);
        return SnowWorker::new(options);
    }

    pub fn set_options(&mut self, options: IdGeneratorOptions) {
        // 1.BaseTime
        if options.base_time == 0 {
            self.base_time = 1582136402000;
        } else if options.base_time < 631123200000
            || options.base_time > Utc::now().timestamp_millis()
        {
            panic!("BaseTime error.");
        } else {
            self.base_time = options.base_time;
        }

        // 2.WorkerIdBitLength
        if options.worker_id_bit_length <= 0 {
            panic!("WorkerIdBitLength error.(range:[1, 21])");
        }
        if options.seq_bit_length + options.worker_id_bit_length > 22 {
            panic!("error：WorkerIdBitLength + SeqBitLength <= 22");
        } else {
            // self.WorkerIdBitLength = options.WorkerIdBitLength;
            self.worker_id_bit_length = if options.worker_id_bit_length <= 0 {
                6
            } else {
                options.worker_id_bit_length
            };
        }

        // 3.WorkerId
        let mut max_worker_id_number = (1 << options.worker_id_bit_length) - 1;
        if max_worker_id_number == 0 {
            max_worker_id_number = 63;
        }
        if options.worker_id > max_worker_id_number {
            panic!("WorkerId error. (range:[0, {} ]", max_worker_id_number);
        } else {
            self.worker_id = options.worker_id;
        }

        // 4.SeqBitLength
        if options.seq_bit_length < 2 || options.seq_bit_length > 21 {
            panic!("SeqBitLength error. (range:[2, 21])");
        } else {
            // self.SeqBitLength = options.SeqBitLength;
            self.seq_bit_length = if options.seq_bit_length <= 0 {
                6
            } else {
                options.seq_bit_length
            };
        }

        // 5.MaxSeqNumber
        let mut max_seq_number = (1 << options.seq_bit_length) - 1;
        if options.max_seq_number <= 0 {
            max_seq_number = 63;
        }
        if options.max_seq_number > max_seq_number {
            panic!("MaxSeqNumber error. (range:[1, {}]", max_seq_number);
        } else {
            self.max_seq_number = if options.max_seq_number == 0 {
                max_seq_number
            } else {
                options.max_seq_number
            };
        }

        // 6.MinSeqNumber
        if options.min_seq_number < 5 || options.min_seq_number > max_seq_number {
            panic!("MinSeqNumber error. (range:[5, {}]", max_seq_number);
        } else {
            self.min_seq_number = options.min_seq_number;
            // self.MinSeqNumber = if options.MinSeqNumber <= 0 { 5 } else { options.MinSeqNumber };
        }

        // 7.TopOverCostCount
        //self.TopOverCostCount = if options.TopOverCostCount == 0 { 2000 } else { options.TopOverCostCount };
        if options.top_over_cost_count > 10000 {
            panic!("TopOverCostCount error. (range:[0, 10000]");
        } else {
            self.top_over_cost_count = options.top_over_cost_count;
        }

        // 8.Others
        self.timestamp_shift = self.worker_id_bit_length + self.seq_bit_length;
        self.current_seq_number = self.min_seq_number;

        if options.method == 1 {
            sleep(std::time::Duration::from_millis(500));
        }
    }

    pub fn new(options: IdGeneratorOptions) -> SnowWorker {
        let mut worker = SnowWorker {
            base_time: 1582136402000,
            worker_id_bit_length: 0,
            worker_id: 0,
            seq_bit_length: 0,
            max_seq_number: 0,
            min_seq_number: 0,
            top_over_cost_count: 0,
            timestamp_shift: 0,
            current_seq_number: 0,

            last_time_tick: 0,
            turn_back_time_tick: 0,
            turn_back_index: 0,
            is_over_cost: false,
            over_cost_count_in_one_term: 0,
            gen_count_in_one_term: 0,
            term_index: 0,
        };

        worker.set_options(options);
        return worker;
    }

    pub fn next_id(&mut self) -> i64 {
        // println!("SeqBitLength: {}", self.SeqBitLength);
        if self.is_over_cost {
            self.next_over_cost_id()
        } else {
            self.next_normal_id()
        }
    }

    fn do_gen_id_action(&self, _arg: OverCostActionArg) {}

    fn begin_over_cost_action(&self, _use_time_tick: i64) {}

    fn end_over_cost_action(&mut self, _use_time_tick: i64) {
        // if self._TermIndex > 10000 {
        //     self._TermIndex = 0;
        // }
    }

    fn begin_turn_back_action(&self, _use_time_tick: i64) {}

    fn end_turn_back_action(&self, _use_time_tick: i64) {}

    fn next_over_cost_id(&mut self) -> i64 {
        let current_time_tick = self.get_current_time_tick();

        if current_time_tick > self.last_time_tick {
            self.end_over_cost_action(current_time_tick);

            self.last_time_tick = current_time_tick;
            self.current_seq_number = self.min_seq_number;
            self.is_over_cost = false;
            self.over_cost_count_in_one_term = 0;
            self.gen_count_in_one_term = 0;

            return self.calc_id(self.last_time_tick);
        }

        if self.over_cost_count_in_one_term >= self.top_over_cost_count {
            self.end_over_cost_action(current_time_tick);

            self.last_time_tick = self.get_next_time_tick();
            self.current_seq_number = self.min_seq_number;
            self.is_over_cost = false;
            self.over_cost_count_in_one_term = 0;
            self.gen_count_in_one_term = 0;

            return self.calc_id(self.last_time_tick);
        }

        if self.current_seq_number > self.max_seq_number {
            self.last_time_tick += 1;
            self.current_seq_number = self.min_seq_number;
            self.is_over_cost = true;
            self.over_cost_count_in_one_term += 1;
            self.gen_count_in_one_term += 1;

            return self.calc_id(self.last_time_tick);
        }

        self.gen_count_in_one_term += 1;
        return self.calc_id(self.last_time_tick);
    }

    fn next_normal_id(&mut self) -> i64 {
        let current_time_tick = self.get_current_time_tick();

        if current_time_tick < self.last_time_tick {
            if self.turn_back_time_tick < 1 {
                self.turn_back_time_tick = self.last_time_tick - 1;
                self.turn_back_index += 1;
                // 每毫秒序列数的前5位是预留位，0用于手工新值，1-4是时间回拨次序
                // 支持4次回拨次序（避免回拨重叠导致ID重复），可无限次回拨（次序循环使用）。
                if self.turn_back_index > 4 {
                    self.turn_back_index = 1;
                }
                self.begin_turn_back_action(self.turn_back_time_tick);
            }

            // thread::sleep(std::time::Duration::from_millis(1));
            return self.calc_turn_back_id(self.turn_back_time_tick);
        }

        // 时间追平时，_TurnBackTimeTick清零
        if self.turn_back_time_tick > 0 {
            self.end_turn_back_action(self.turn_back_time_tick);
            self.turn_back_time_tick = 0;
        }

        if current_time_tick > self.last_time_tick {
            self.last_time_tick = current_time_tick;
            self.current_seq_number = self.min_seq_number;

            return self.calc_id(self.last_time_tick);
        }

        if self.current_seq_number > self.max_seq_number {
            self.begin_over_cost_action(current_time_tick);

            self.term_index += 1;
            self.last_time_tick += 1;
            self.current_seq_number = self.min_seq_number;
            self.is_over_cost = true;
            self.over_cost_count_in_one_term = 1;
            self.gen_count_in_one_term = 1;

            return self.calc_id(self.last_time_tick);
        }

        return self.calc_id(self.last_time_tick);
    }

    fn calc_id(&mut self, use_time_tick: i64) -> i64 {
        let result = (use_time_tick << self.timestamp_shift)
            + (self.worker_id << self.seq_bit_length) as i64
            + (self.current_seq_number) as i64;
        self.current_seq_number += 1;
        return result;
    }

    fn calc_turn_back_id(&mut self, use_time_tick: i64) -> i64 {
        let result = (use_time_tick << self.timestamp_shift)
            + (self.worker_id << self.seq_bit_length) as i64
            + (self.turn_back_index) as i64;
        self.turn_back_time_tick -= 1;
        return result;
    }

    fn get_current_time_tick(&self) -> i64 {
        return Utc::now().timestamp_millis() - self.base_time;
    }

    fn get_next_time_tick(&self) -> i64 {
        let mut temp_time_ticker = self.get_current_time_tick();

        while temp_time_ticker <= self.last_time_tick {
            // 暂停1ms
            sleep(std::time::Duration::from_millis(1));
            temp_time_ticker = self.get_current_time_tick();
        }

        return temp_time_ticker;
    }
}
