use time::OffsetDateTime;
use volo_gen::google::protobuf::Timestamp;

pub fn db_time_to_proto_time(db_time: OffsetDateTime) -> Timestamp {
    let seconds = db_time.unix_timestamp();
    let nanos = db_time.nanosecond();
    Timestamp {
        seconds,
        nanos: nanos.try_into().unwrap(),
    }
}
