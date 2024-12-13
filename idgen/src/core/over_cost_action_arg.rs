pub struct OverCostActionArg {
    action_type: u32,
    time_tick: i64,
    worker_id: u16,
    over_cost_count_in_one_term: i32,
    gen_count_in_one_term: i32,
    term_index: i32,
}