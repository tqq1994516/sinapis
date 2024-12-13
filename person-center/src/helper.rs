use entity::entities::user_property::Model as User;
use volo_gen::person_center::UserResponse;
// use utils::data_handle::db_time_to_proto_time;
use volo_gen::google::protobuf::{Any, Timestamp};

pub fn user_model_to_user_grpc_response(user: User) -> UserResponse {
    let info = Any {
        type_url: "".to_owned().into(),
        value: user.extra.unwrap().to_string().into_bytes().into(),
    };
    let create_time = db_time_to_proto_time(user.create_time.unwrap());
    let update_time = db_time_to_proto_time(user.update_time.unwrap());
    let latest_login_time = db_time_to_proto_time(user.latest_login_time.unwrap());
    let seconds = user.birthday.unwrap().and_hms_micro_opt(0, 0, 0, 0).unwrap().timestamp();
    let birthday = Timestamp {
        seconds,
        nanos: 0,
    };
    UserResponse {
        id: user.id,
        name: user.username.into(),
        password: user.password.into(),
        email: user.email.unwrap().into(),
        phone: user.phone.unwrap().into(),
        online: user.online.is_some(),
        info: Some(info),
        create_time: Some(create_time),
        update_time: Some(update_time),
        organization: 1,
        first_name: user.first_name.unwrap().into(),
        last_name: user.last_name.unwrap().into(),
        birthday: Some(birthday),
        gender: user.gender.unwrap().to_str().into(),
        latest_login_time: Some(latest_login_time),
    }
}