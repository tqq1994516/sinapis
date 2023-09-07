use prost_types::Any;
use prost::Message;

use entity::user_info::Model as User;
use person_center::UserResponse;
use utils::data_handle::db_time_to_proto_time;

pub mod person_center {
    tonic::include_proto!("person_center");
}

pub fn user_model_to_user_grpc_response(user: User) -> UserResponse {
    let info = Any {
        type_url: "".to_owned(),
        value: user.info.unwrap().to_string().into_bytes(),
    };
    let create_time = db_time_to_proto_time(user.create_time.unwrap());
    let update_time = db_time_to_proto_time(user.update_time.unwrap());
    UserResponse {
        id: user.id,
        name: user.name,
        password: user.password,
        email: user.email.unwrap(),
        phone: user.phone.unwrap(),
        online: user.online.is_some(),
        info: Some(info),
        create_time: Some(create_time),
        update_time: Some(update_time),
    }
}