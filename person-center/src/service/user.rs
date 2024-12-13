use pilota::AHashMap;
use sea_orm::{
    prelude::*,
    ActiveValue::{NotSet, Set},
    Condition,
};
use serde_json::json;
use volo_grpc::{Code, Response, Status};

use entity::{
    graph::{create_node, search_node, NodeType, NodeTypeObject, User, VertexTypeObject},
    user_property,
};
use pool::age::Client;
use utils::{encryption::encryption, extra_to_outer};
use volo_gen::person_center::{
    FilterUserRequest, Logged, LoginForm, PrivateUserInfo, UserInfo, UserResponse, UsersResponse,
};

pub async fn handler_add_user(
    body: PrivateUserInfo,
    db: &DatabaseConnection,
    age_client: &Client,
) -> Result<Response<UserResponse>, Status> {
    let user_name = body.name;

    let mut properties = body.extra.clone();
    // 插入graph user
    properties.insert(
        "alias".into(),
        body.alias.clone().or(Some("".into())).unwrap(),
    );
    properties.insert(
        "email".into(),
        body.email.clone().or(Some("".into())).unwrap(),
    );
    properties.insert(
        "phone".into(),
        body.phone.clone().or(Some("".into())).unwrap(),
    );

    // 查询user是否已存在
    match search_node(
        age_client,
        NodeType::User,
        Some(&user_name),
        None,
        properties.clone(),
    )
    .await
    {
        Ok(_) => {
            return Err(Status::already_exists(format!(
                "user: {} exists!",
                user_name
            )))
        }
        Err(e) => {
            // 不存在则进行插入
            if e.code() != Code::NotFound {
                return Err(e);
            }
        }
    };

    let user = User {
        name: user_name.to_string(),
        properties: properties.clone(),
    };
    if let Some(s) = create_node(age_client, NodeTypeObject::User(user)).await {
        return Err(s);
    }

    // 查询插入结果
    let node = match search_node(
        age_client,
        NodeType::User,
        Some(&user_name),
        None,
        properties.clone(),
    )
    .await
    {
        Ok(node) => node,
        Err(s) => return Err(s),
    };
    if let VertexTypeObject::User(node) = node {
        let extra = match serde_json::to_string(&body.extra) {
            Ok(extra) => extra,
            Err(e) => return Err(Status::from_error(Box::new(e))),
        };
        // 获取user id 插入关联表
        let user_property = user_property::ActiveModel {
            id: Set(node.id() as i64),
            name: Set(user_name.to_owned().into()),
            alias: Set(body.alias.clone().map(Into::into)),
            email: Set(body.email.clone().map(Into::into)),
            phone: Set(body.phone.clone().map(Into::into)),
            extra: Set(Some(json!(extra))),
            created_at: NotSet,
            updated_at: NotSet,
            password: Set(encryption(&body.password).as_bytes().to_vec()),
        };
        if let Err(e) = user_property.insert(db).await {
            return Err(Status::from_error(Box::new(e)));
        }

        let user_info = Some(UserInfo {
            name: user_name.clone(),
            alias: body.alias,
            email: body.email,
            phone: body.phone,
            extra: body.extra,
        });
        Ok(Response::new(UserResponse {
            id: node.id() as i64,
            user: user_info,
        }))
    } else {
        Err(Status::aborted("node type error!"))
    }
}

pub async fn handler_search_user(
    body: FilterUserRequest,
    db: &DatabaseConnection,
    age_client: &Client,
) -> Result<Response<UsersResponse>, Status> {
    let mut conditions = Condition::all();
    if let Some(email) = body.email {
        conditions = conditions.add(user_property::Column::Email.starts_with(email.to_string()));
    }
    if let Some(phone) = body.phone {
        conditions = conditions.add(user_property::Column::Phone.starts_with(phone.to_string()));
    }
    if let Some(id) = body.id {
        conditions = conditions.add(user_property::Column::Id.eq(id));
    }
    let users = match user_property::Entity::find()
        .filter(conditions)
        .all(db)
        .await
    {
        Ok(users) => users,
        Err(e) => return Err(Status::from_error(Box::new(e))),
    };

    if users.is_empty() {
        return Err(Status::not_found("User not found!"));
    }

    let mut users_res: Vec<UserResponse> = Vec::new();
    for user in users.iter() {
        let extra = extra_to_outer(user.clone().extra);
        let alias = user.clone().alias.map(Into::into);
        let email = user.clone().email.map(Into::into);
        let phone = user.clone().phone.map(Into::into);
        let name = user.clone().name;
        // 查询graph结果
        if let Ok(_) = search_node(
            age_client,
            NodeType::User,
            Some(&name),
            Some(user.id),
            extra.clone(),
        )
        .await
        {
            let user_info = Some(UserInfo {
                name: name.into(),
                alias,
                email,
                phone,
                extra,
            });
            users_res.push(UserResponse {
                id: user.id,
                user: user_info,
            });
        }
    }
    Ok(Response::new(UsersResponse { users: users_res }))
}

pub async fn handler_login(
    body: LoginForm,
    db: &DatabaseConnection,
    age_client: &Client,
) -> Result<Response<Logged>, Status> {
    let user_name = body.username;
    // 查询user是否已存在
    match search_node(
        age_client,
        NodeType::User,
        Some(&user_name),
        None,
        AHashMap::new(),
    )
    .await
    {
        Ok(ov) => {
            if let VertexTypeObject::User(user) = ov {
                // 查询user关联表
                match user_property::Entity::find_by_id(user.id() as i64)
                    .one(db)
                    .await
                {
                    Ok(user) => match user {
                        Some(u) => {
                            if encryption(&body.password).as_bytes().to_vec() == u.password {
                                Ok(Response::new(Logged {
                                    access_token: "access_token".into(),
                                    refresh_token: "refresh_token".into(),
                                }))
                            } else {
                                Err(Status::aborted("password check fail!"))
                            }
                        }
                        None => Err(Status::not_found("User not found!")),
                    },
                    Err(e) => Err(Status::from_error(Box::new(e))),
                }
            } else {
                Err(Status::aborted("node type error!"))
            }
        }
        Err(e) => Err(e),
    }
}
