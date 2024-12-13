use pilota::AHashMap;
use volo::FastStr;
use volo_grpc::{Code, Response, Status};
use apache_age::Vertex;

use entity::{
    graph::{
        assignment, create_node, search_node, search_user_attribute_node, Assignment, NodeType,
        NodeTypeObject, UserAttribute, VertexTypeObject,
    },
    user_property,
};
use sea_orm::{DatabaseConnection, EntityTrait};
use volo_gen::person_center::{
    AddUserAttributeRequest, FilterAttributeRequest, UserAttributeInfo,
    UserAttributeOriginNodeType, UserAttributeResponse, UserAttributesResponse,
};
use pool::age::Client;

pub async fn handler_add_user_attribute(
    body: AddUserAttributeRequest,
    db: &DatabaseConnection,
    age_client: &Client,
) -> Result<Response<UserAttributeResponse>, Status> {
    // 查询是否已存在
    match search_node(
        age_client,
        NodeType::UserAttribute,
        Some(&body.name),
        None,
        body.properties.clone(),
    )
    .await
    {
        Ok(vo) => {
            if let VertexTypeObject::UserAttribute(ua) = vo {
                return Err(Status::already_exists(format!(
                    "user attribute: {} exists!",
                    ua.properties().name,
                )));
            } else {
                return Err(Status::aborted("node type error!"));
            }
        }
        Err(e) => {
            if e.code() != Code::NotFound {
                return Err(e);
            }
        }
    };
    if body.origin_node_type == UserAttributeOriginNodeType::USER {
        let origin = user_property::Entity::find_by_id(body.origin_id)
            .one(db)
            .await
            .map_err(|e| Status::from_error(Box::new(e)))?;

        // 查找原始origin节点
        match origin {
            None => {
                return Err(Status::not_found(format!(
                    "origin id: {} not found",
                    body.origin_id
                )))
            }
            Some(origin) => {
                match search_node(
                    age_client,
                    NodeType::User,
                    Some(&origin.name),
                    Some(origin.id),
                    AHashMap::new(),
                )
                .await
                {
                    Ok(vo) => {
                        if let VertexTypeObject::User(u) = vo {
                            let node =
                                insert_user_attribute(age_client, body.name.to_string(), body.properties)
                                    .await?;
                            handler_assignment_combination(
                                age_client,
                                node,
                                u.id() as i64,
                                body.origin_node_type,
                                AHashMap::new(),
                            )
                            .await
                        } else {
                            Err(Status::aborted("node type error!"))
                        }
                    }
                    Err(e) => Err(e),
                }
            }
        }
    } else {
        match search_node(
            age_client,
            NodeType::UserAttribute,
            None,
            Some(body.origin_id),
            AHashMap::new(),
        )
        .await
        {
            Ok(vo) => {
                if let VertexTypeObject::UserAttribute(ua) = vo {
                    let node =
                        insert_user_attribute(age_client, body.name.to_string(), body.properties).await?;
                    handler_assignment_combination(
                        age_client,
                        node,
                        ua.id() as i64,
                        body.origin_node_type,
                        AHashMap::new(),
                    )
                    .await
                } else {
                    Err(Status::aborted("node type error!"))
                }
            }
            Err(e) => Err(e),
        }
    }
}

pub async fn handler_search_user_attribute(
    body: FilterAttributeRequest,
    age_client: &Client,
) -> Result<Response<UserAttributesResponse>, Status> {
    let mut user_attributes_res: Vec<UserAttributeResponse> = Vec::new();
    match search_user_attribute_node(age_client, body.target_id, body.name.as_deref(), body.properties)
        .await
    {
        Ok(uas) => {
            for uar in uas.iter() {
                let ua: Vertex<UserAttribute> = uar.get(0);
                user_attributes_res.push(UserAttributeResponse {
                    id: ua.id() as i64,
                    user_attribute: Some(UserAttributeInfo {
                        name: ua.properties().name.clone().into(),
                        extra: ua.properties().properties.clone(),
                    }),
                });
            }
        }
        Err(e) => return Err(e),
    }
    Ok(Response::new(UserAttributesResponse { user_attributes: user_attributes_res }))
}

// pub async fn handler_edit_user_attribute(
//     body: EditUserAttributeRequest,
//     db: &DatabaseConnection,
// ) -> Result<Response<UserAttributeResponse>, Status> {
//     // 查询是否已存在
//     match search_node(
//         NodeType::UserAttribute,
//         Some(&body.name),
//         Some(body.user_attribute_id),
//         body.properties.clone(),
//     )
//     .await
//     {
//         Ok(vo) => {
//             if let VertexTypeObject::UserAttribute(ua) = vo {
//                 return Err(Status::already_exists(format!(
//                     "user attribute: {} exists!",
//                     ua.properties().name,
//                 )));
//             } else {
//                 return Err(Status::aborted("node type error!"));
//             }
//         }
//         Err(e) => return Err(Status::from_error(Box::new(e)));
//     };
// }

async fn insert_user_attribute(
    client: &Client,
    name: String,
    properties: AHashMap<FastStr, FastStr>,
) -> Result<VertexTypeObject, Status> {
    let user_attribute = UserAttribute {
        name: name.clone(),
        properties: properties.clone(),
    };
    if let Some(s) = create_node(client, NodeTypeObject::UserAttribute(user_attribute)).await {
        return Err(s);
    }

    // 查询插入结果
    let node = match search_node(
        client,
        NodeType::UserAttribute,
        Some(&name),
        None,
        properties.clone(),
    )
    .await
    {
        Ok(node) => node,
        Err(s) => return Err(s),
    };
    Ok(node)
}

async fn handler_assignment_combination(
    client: &Client,
    node: VertexTypeObject,
    origin_id: i64,
    origin_node_type: UserAttributeOriginNodeType,
    properties: AHashMap<FastStr, FastStr>,
) -> Result<Response<UserAttributeResponse>, Status> {
    if let VertexTypeObject::UserAttribute(node) = node {
        // 添加指派关系
        let mut assignment_combination = Assignment::U2UA((origin_id, node.id() as i64));
        if origin_node_type == UserAttributeOriginNodeType::USER_ATTRIBUTE {
            assignment_combination = Assignment::UA2UA((origin_id, node.id() as i64));
        }
        if let Some(e) = assignment(client, assignment_combination).await {
            return Err(e);
        };

        let user_attribute = Some(UserAttributeInfo {
            name: node.properties().name.clone().into(),
            extra: properties,
        });
        Ok(Response::new(UserAttributeResponse {
            id: node.id() as i64,
            user_attribute,
        }))
    } else {
        Err(Status::aborted("node type error!"))
    }
}
