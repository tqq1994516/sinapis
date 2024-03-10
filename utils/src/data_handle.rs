use chrono::{DateTime, FixedOffset};

use volo_gen::google::protobuf::Timestamp;

pub fn db_time_to_proto_time(db_time: DateTime<FixedOffset>) -> Timestamp {
    Timestamp {
        seconds: db_time.timestamp(),
        nanos: db_time.timestamp_subsec_nanos() as i32,
    }
}
