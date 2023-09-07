use std::error::Error;
use async_recursion::async_recursion;
use sea_orm::{
    Condition,
    QueryOrder,
    entity::prelude::*,
    ActiveValue::{
        Set,
        NotSet,
    },
};
use dapr::appcallback::InvokeResponse;
use prost::Message;
use prost_types::Any;
use time::OffsetDateTime;

use entity::prelude::*;
use entity::*;
use super::utils::user_model_to_user_grpc_response;
use utils::encryption::encryption;
use person_center::*;
use report::*;

pub mod person_center {
    tonic::include_proto!("person_center");
}

pub mod report {
    tonic::include_proto!("report");
}


pub async fn user_detail(
    data: &Vec<u8>,
    db: &DatabaseConnection
) -> Result<InvokeResponse, Box<dyn Error>> {
    let req = UserDetail::decode(&data[..])?;
    let user = UserInfo::find_by_id(req.id).one(db).await?;
    match user {
        Some(value) => {
            let resp = user_model_to_user_grpc_response(value);
            let data = resp.encode_to_vec();
            let data = Any {
                type_url: "".to_owned(),
                value: data,
            };
            let invoke_response = InvokeResponse {
                content_type: "application/json".to_string(),
                data: Some(data),
            };
            Ok(invoke_response)
        },
        None => Ok(InvokeResponse::default()),
    }
}

pub async fn user_list(
    data: &Vec<u8>,
    db: &DatabaseConnection
) -> Result<InvokeResponse, Box<dyn Error>> {
    let req = UserList::decode(&data[..])?;
    let mut condition = Condition::all();
    if req.id.is_some() {
        condition = condition.add(user_info::Column::Id.eq(req.id.unwrap()));
    }
    if req.name.is_some() {
        condition = condition.add(user_info::Column::Name.eq(req.name.unwrap()));
    }
    let users = UserInfo::find()
        .filter(condition)
        .order_by_desc(user_info::Column::Id)
        .all(db)
        .await?;
    let users = users.into_iter()
        .map(|u| user_model_to_user_grpc_response(u))
        .collect::<Vec<UserResponse>>();
    let resp = UsersResponse { users: users.into() };
    let data = resp.encode_to_vec();
    let data = prost_types::Any {
        type_url: "".to_owned(),
        value: data,
    };
    let invoke_response = InvokeResponse {
        content_type: "application/json".to_string(),
        data: Some(data),
    };
    Ok(invoke_response)
}

pub async fn update_user(
    data: &Vec<u8>,
    db: &DatabaseConnection
) -> Result<InvokeResponse, Box<dyn Error>> {
    let req = UpdateUser::decode(&data[..])?;
    let user: Option<user_info::Model> = UserInfo::find_by_id(req.id).one(db).await?;
    match user {
        Some(value) => {
            let id = value.id;
            let info = match &req.info {
                Some(info) => Set(Some(serde_json::from_slice(&info.value)?)),
                None => Set(Some(value.info.unwrap())),
            };
            let new_user = user_info::ActiveModel {
                id: Set(id),
                name: Set(req.name.unwrap_or(String::from(&value.name))),
                email: Set(Some(req.email.unwrap_or(String::from(value.email.unwrap())))),
                phone: Set(Some(req.phone.unwrap_or(String::from(value.phone.unwrap())))),
                info,
                update_time: Set(Some(OffsetDateTime::now_utc())),
                password: Set(encryption(&req.password.unwrap_or(value.password))),
                online: Set(value.online),
                create_time: Set(value.create_time),
                organization: Set(value.organization),
                accessible: Set(value.accessible),
                period_of_validity: Set(value.period_of_validity),
                available: Set(value.available),
            };
            let user = UserInfo::update(new_user)
                .filter(user_info::Column::Id.eq(id))
                .exec(db)
                .await?;
            let resp = user_model_to_user_grpc_response(user);
            let data = resp.encode_to_vec();
            let data = prost_types::Any {
                type_url: "".to_owned(),
                value: data,
            };
            let invoke_response = InvokeResponse {
                content_type: "application/json".to_string(),
                data: Some(data),
            };
            Ok(invoke_response)
        },
        None => Ok(InvokeResponse::default()),
    }
}

pub async fn insert_user(
    data: &Vec<u8>,
    db: &DatabaseConnection
) -> Result<InvokeResponse, Box<dyn Error>> {
    let req = InsertUser::decode(&data[..])?;
    if let Some(info) = req.info {
        let info_value = serde_json::from_slice(&info.value)?;
        let user = user_info::ActiveModel {
            id: NotSet,
            name: Set(req.name),
            password: Set(encryption(&req.password)),
            email: Set(req.email),
            phone: Set(req.phone),
            online: Set(Some(false)),
            info: Set(Some(info_value)),
            create_time: Set(Some(OffsetDateTime::now_utc())),
            update_time: Set(Some(OffsetDateTime::now_utc())),
            organization: Set(req.organization),
            accessible: Set(true),
            period_of_validity: NotSet,
            available: Set(true),
        };
        let user = user.insert(db).await?;
        let resp = user_model_to_user_grpc_response(user);
        let data = resp.encode_to_vec();
        let data = prost_types::Any {
            type_url: "".to_owned(),
            value: data,
        };
        let invoke_response = InvokeResponse {
            content_type: "application/json".to_string(),
            data: Some(data),
        };
        Ok(invoke_response)
    } else {
        Ok(InvokeResponse::default())
    }
}

pub async fn delete_user(
    data: &Vec<u8>,
    db: &DatabaseConnection
) -> Result<InvokeResponse, Box<dyn Error>> {
    let req = UserDetail::decode(&data[..])?;
    let user = UserInfo::delete_by_id(req.id).exec(db).await?;
    let resp = Report {
        message: format!("user id : {:#?} is delete.", user),
    };
    let data = resp.encode_to_vec();
    let data = prost_types::Any {
        type_url: "".to_owned(),
        value: data,
    };
    let invoke_response = InvokeResponse {
        content_type: "application/json".to_string(),
        data: Some(data),
    };
    return Ok(invoke_response);
}

// 在evaluate时考虑继承
async fn evaluate(
    db: &DatabaseConnection,
    user: &user_info::Model,
    permissions: &[permission::Model],
    policies: &[policy::Model]
) -> Result<bool, Box<dyn Error>> {
    let mut allow = false;

    // 递归收集主体所有角色
    let roles = user.find_related(Role).all(db).await?;
    let mut role_list = Vec::new();
    role_list.extend(roles.clone());
    let role_list = collect_roles(db, roles, &mut role_list).await?; 


    // 检查角色 permissions及policies
    let role_permissions: Vec<Vec<permission::Model>> = role_list.load_many_to_many(Permission, RolePermission, db).await?;
    for permissin_list in role_permissions {
        for permission in permissin_list {
            allow = permissions.iter().any(|p| p.id == permission.id);
        }
        
    }
    let role_policies: Vec<Vec<policy::Model>> = role_list.load_many_to_many(Policy, RolePolicy, db).await?;
    for policy_list in role_policies {
        for policy in policy_list {
            allow = policies.iter().any(|p| p.id == policy.id);
        }
    }
    Ok(allow)
}

// 递归收集角色继承关系，在权限检查时递归合并父角色权限
#[async_recursion]
async fn collect_roles<'a>(
    db: &'a DatabaseConnection,
    roles: Vec<role::Model>,
    role_list: &'a mut Vec<role::Model>
) -> Result<&'a mut Vec<role::Model>, Box<dyn Error>> {
    for r in roles {
        let mut parents = r.find_linked(role::ChildParent).all(db).await?;
        role_list.append(&mut parents);
        let mut role_list_copy = role_list.clone();
        let nested_parents = collect_roles(db, parents, &mut role_list_copy).await?;
        role_list.append(nested_parents);
    }
    role_list.dedup();
    Ok(role_list)
}
