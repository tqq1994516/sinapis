use sea_orm::DatabaseConnection;
use std::error::Error;
use sea_orm::{
    Condition,
    QueryOrder,
    entity::prelude::*,
    ActiveValue::{
        Set,
        NotSet,
    },
};
use time::OffsetDateTime;

use entity::{*, prelude::*};
use volo_gen::person_center::{
    UsersResponse,
    UserResponse,
    Report,
    UserListReq,
    UserDetailReq,
    UpdateUserReq,
    InsertUserReq,
    LoginForm,
    Logged,
    CheckPermissionReq,
    Accessable,
};
use utils::encryption::encryption;
use super::helper::user_model_to_user_grpc_response;

pub async fn user_detail_service(
    data: UserDetailReq,
    db: &DatabaseConnection
) -> Result<UserResponse, Box<dyn Error>> {
    let user = UserInfo::find_by_id(data.id).one(db).await?;
    match user {
        Some(value) => {
            Ok(user_model_to_user_grpc_response(value))
        },
        None => Ok(UserResponse::default()),
    }
}

pub async fn user_list_service(
    data: UserListReq,
    db: &DatabaseConnection
) -> Result<UsersResponse, Box<dyn Error>> {
    let mut condition = Condition::all();
    if data.id.is_some() {
        condition = condition.add(user_info::Column::Id.eq(data.id.unwrap()));
    }
    if data.name.is_some() {
        condition = condition.add(user_info::Column::Name.eq(String::from(data.name.unwrap())));
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
    Ok(resp)
}

pub async fn update_user_service(
    data: UpdateUserReq,
    db: &DatabaseConnection
) -> Result<UserResponse, Box<dyn Error>> {
    let user: Option<user_info::Model> = UserInfo::find_by_id(data.id).one(db).await?;
    match user {
        Some(value) => {
            let id = value.id;
            let info = match &data.info {
                Some(info) => Set(Some(serde_json::from_slice(&info.value)?)),
                None => Set(Some(value.info.unwrap())),
            };
            let new_user = user_info::ActiveModel {
                id: Set(id),
                name: Set(data.name.unwrap_or(String::from(&value.name).into()).to_string()),
                email: Set(Some(data.email.unwrap_or(String::from(value.email.unwrap()).into()).to_string())),
                phone: Set(Some(data.phone.unwrap_or(String::from(value.phone.unwrap()).into()).to_string())),
                info,
                update_time: Set(Some(OffsetDateTime::now_utc())),
                password: Set(encryption(&data.password.unwrap_or(value.password.into()))),
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
            Ok(user_model_to_user_grpc_response(user))
        },
        None => Ok(UserResponse::default()),
    }
}

pub async fn insert_user_service(
    data: InsertUserReq,
    db: &DatabaseConnection
) -> Result<UserResponse, Box<dyn Error>> {
    if let Some(info) = data.info {
        let info_value = serde_json::from_slice(&info.value)?;
        let user = user_info::ActiveModel {
            id: NotSet,
            name: Set(data.name.to_string()),
            password: Set(encryption(&data.password)),
            email: Set(Some(data.email.unwrap_or("".into()).to_string())),
            phone: Set(Some(data.phone.unwrap_or("".into()).to_string())),
            online: Set(Some(false)),
            info: Set(Some(info_value)),
            create_time: Set(Some(OffsetDateTime::now_utc())),
            update_time: Set(Some(OffsetDateTime::now_utc())),
            organization: Set(data.organization),
            accessible: Set(true),
            period_of_validity: NotSet,
            available: Set(true),
        };
        let user = user.insert(db).await?;
        Ok(user_model_to_user_grpc_response(user))
    } else {
        Ok(UserResponse::default())
    }
}

pub async fn delete_user_service(
    data: UserDetailReq,
    db: &DatabaseConnection
) -> Result<Report, Box<dyn Error>> {
    let user = UserInfo::delete_by_id(data.id).exec(db).await?;
    let resp = Report {
        message: format!("user id : {:#?} is delete.", user).into(),
    };
    Ok(resp)
}

pub async fn login_service(
    data: LoginForm,
    db: &DatabaseConnection
) -> Result<Logged, Box<dyn Error>> {
    let user = UserInfo::find()
        .filter(
            Condition::all()
                .add(
                    user_info::Column::Name.eq(data.username.to_string())
                )
                .add(
                    user_info::Column::Password.eq(encryption(&data.password))
                )
            )
            .all(db)
            .await?;
    let resp = Logged {
        access_token: "1111".to_owned().into(),
        refresh_token: "22222".to_owned().into(),
    };
    Ok(resp)
}

pub async fn check_permission_service(
    data: CheckPermissionReq,
    db: &DatabaseConnection
) -> Result<Accessable, Box<dyn Error>> {
    Ok(Accessable { accessable: true })
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
#[pilota::async_recursion::async_recursion]
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
