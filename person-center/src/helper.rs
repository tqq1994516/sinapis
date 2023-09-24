use entity::user_info::Model as User;
use volo_gen::person_center::UserResponse;
use utils::data_handle::db_time_to_proto_time;
use volo_gen::google::protobuf::Any;

pub fn user_model_to_user_grpc_response(user: User) -> UserResponse {
    let info = Any {
        type_url: "".to_owned().into(),
        value: user.info.unwrap().to_string().into_bytes().into(),
    };
    let create_time = db_time_to_proto_time(user.create_time.unwrap());
    let update_time = db_time_to_proto_time(user.update_time.unwrap());
    UserResponse {
        id: user.id,
        name: user.name.into(),
        password: user.password.into(),
        email: user.email.unwrap().into(),
        phone: user.phone.unwrap().into(),
        online: user.online.is_some(),
        info: Some(info),
        create_time: Some(create_time),
        update_time: Some(update_time),
    }
}