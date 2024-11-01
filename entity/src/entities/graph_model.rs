use apache_age::{tokio::AgeClient, Vertex};
use pilota::{AHashMap, FastStr};
use sonic_rs::{Deserialize, Serialize};
use std::fmt::Display;
use volo_grpc::Status;

use crate::get_age;

pub const GRAPH_NAME: &str = "ngac";
pub const CREATE: &str = "CREATE";
pub const MATCH: &str = "MATCH";
pub const SET: &str = "SET";
pub const WHERE: &str = "WHERE";
pub const AND: &str = "AND";
pub const RETURN: &str = "RETURN";
pub const ASSOCIATION: &str = "Association";

pub struct OpenCypherFunc;

impl OpenCypherFunc {
    pub fn id(node: &str) -> String {
        format!("id({})", node)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub name: String,
    pub properties: AHashMap<FastStr, FastStr>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserAttribute {
    pub name: String,
    pub properties: AHashMap<FastStr, FastStr>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Object {
    pub name: String,
    pub properties: AHashMap<FastStr, FastStr>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ObjectAttribute {
    pub name: String,
    pub properties: AHashMap<FastStr, FastStr>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PolicyClass {
    pub name: String,
    pub properties: AHashMap<FastStr, FastStr>,
}

pub enum Assignment {
    U2UA((i64, i64)),
    UA2UA((i64, i64)),
    UA2OA((i64, i64)),
    UA2PC((i64, i64)),
    O2OA((i64, i64)),
    OA2OA((i64, i64)),
    OA2PC((i64, i64)),
}

#[derive(Clone)]
pub enum NodeType {
    User,
    UserAttribute,
    Object,
    ObjectAttribute,
    PolicyClass,
}

impl Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeType::User => write!(f, "u"),
            NodeType::UserAttribute => write!(f, "ua"),
            NodeType::Object => write!(f, "o"),
            NodeType::ObjectAttribute => write!(f, "oa"),
            NodeType::PolicyClass => write!(f, "pc"),
        }
    }
}

impl NodeType {
    fn fmt_full(&self) -> String {
        match self {
            NodeType::User => "User".to_owned(),
            NodeType::UserAttribute => "UserAttribute".to_owned(),
            NodeType::Object => "Object".to_owned(),
            NodeType::ObjectAttribute => "ObjectAttribute".to_owned(),
            NodeType::PolicyClass => "PolicyClass".to_owned(),
        }
    }
}

pub enum NodeTypeObject {
    User(User),
    UserAttribute(UserAttribute),
    Object(Object),
    ObjectAttribute(ObjectAttribute),
    PolicyClass(PolicyClass),
}

pub enum VertexTypeObject {
    User(Vertex<User>),
    UserAttribute(Vertex<UserAttribute>),
    Object(Vertex<Object>),
    ObjectAttribute(Vertex<ObjectAttribute>),
    PolicyClass(Vertex<PolicyClass>),
}

fn properties_to_query_condition(
    node_type: &NodeType,
    properties: AHashMap<FastStr, FastStr>,
) -> String {
    let mut condition = String::from("");
    if properties.is_empty() {
        condition
    } else {
        for (k, v) in properties.iter() {
            condition.push_str(&format!(" {} {}.{} = '{}'", AND, node_type, k, v));
        }
        condition
    }
}

fn properties_to_property_query_condition(
    properties: AHashMap<FastStr, FastStr>,
) -> Option<String> {
    if properties.is_empty() {
        None
    } else {
        let mut conditions = Vec::new();
        for (k, v) in properties.iter() {
            conditions.push(format!("{}: '{}'", k, v));
        }
        Some(format!(" {} ", conditions.join(", ")))
    }
}

pub async fn create_node(node: NodeTypeObject) -> Option<Status> {
    let age = match get_age().await {
        Ok(age) => age,
        Err(e) => return Some(Status::from_error(e)),
    };

    let (node_type, query_condition) = match node {
        NodeTypeObject::User(user) => (
            NodeType::User,
            properties_to_property_query_condition(user.properties).unwrap(),
        ),
        NodeTypeObject::UserAttribute(user_attribute) => (
            NodeType::UserAttribute,
            properties_to_property_query_condition(user_attribute.properties).unwrap(),
        ),
        NodeTypeObject::Object(object) => (
            NodeType::Object,
            properties_to_property_query_condition(object.properties).unwrap(),
        ),
        NodeTypeObject::ObjectAttribute(object_attribute) => (
            NodeType::ObjectAttribute,
            properties_to_property_query_condition(object_attribute.properties).unwrap(),
        ),
        NodeTypeObject::PolicyClass(policy_class) => (
            NodeType::PolicyClass,
            properties_to_property_query_condition(policy_class.properties).unwrap(),
        ),
    };
    let _ = age
        .unique_index(GRAPH_NAME, &node_type.fmt_full(), "unique_name", "name")
        .await
        .unwrap();
    let st = match age
        .prepare_cypher(
            GRAPH_NAME,
            &format!(
                "{} ({}: {} {{{}}}) ",
                CREATE,
                node_type,
                node_type.fmt_full(),
                query_condition
            ),
            false,
        )
        .await
    {
        Ok(st) => st,
        Err(e) => return Some(Status::from_error(Box::new(e))),
    };

    if let Err(e) = age.query(&st, &[]).await {
        return Some(Status::from_error(Box::new(e)));
    }
    None
}

pub fn create_association_cypher(
    origin_node_type: NodeType,
    origin_node_id: i64,
    target_node_type: NodeType,
    target_node_id: i64,
    properties: AHashMap<FastStr, FastStr>,
) -> String {
    match properties_to_property_query_condition(properties) {
        Some(p) => format!(
            "{} ({}: {}), ({}: {}) {} {} = {} {} {} = {} {} ({})-[r:{} {{{}}}]->({}) {} r",
            MATCH,
            origin_node_type,
            origin_node_type.fmt_full(),
            target_node_type,
            target_node_type.fmt_full(),
            WHERE,
            OpenCypherFunc::id(&origin_node_type.to_string()),
            origin_node_id,
            AND,
            OpenCypherFunc::id(&target_node_type.to_string()),
            target_node_id,
            CREATE,
            origin_node_type,
            ASSOCIATION,
            p,
            target_node_type,
            RETURN
        ),
        None => format!(
            "{} ({}: {}), ({}: {}) {} {} = {} {} {} = {} {} ({})-[r:{}]->({}) {} r",
            MATCH,
            origin_node_type,
            origin_node_type.fmt_full(),
            target_node_type,
            target_node_type.fmt_full(),
            WHERE,
            OpenCypherFunc::id(&origin_node_type.to_string()),
            origin_node_id,
            AND,
            OpenCypherFunc::id(&target_node_type.to_string()),
            target_node_id,
            CREATE,
            origin_node_type,
            ASSOCIATION,
            target_node_type,
            RETURN
        ),
    }
}

pub async fn assignment(assignment_combination: Assignment) -> Option<Status> {
    let age = match get_age().await {
        Ok(age) => age,
        Err(e) => return Some(Status::from_error(e)),
    };
    let cypher = match assignment_combination {
        Assignment::U2UA((user_id, user_attribute_id)) => create_association_cypher(
            NodeType::User,
            user_id,
            NodeType::UserAttribute,
            user_attribute_id,
            AHashMap::new(),
        ),
        Assignment::UA2UA((sub_user_attribute_id, user_attribute_id)) => create_association_cypher(
            NodeType::UserAttribute,
            sub_user_attribute_id,
            NodeType::UserAttribute,
            user_attribute_id,
            AHashMap::new(),
        ),
        Assignment::UA2OA((user_attribute_id, object_attribute_id)) => create_association_cypher(
            NodeType::UserAttribute,
            user_attribute_id,
            NodeType::ObjectAttribute,
            object_attribute_id,
            AHashMap::new(),
        ),
        Assignment::UA2PC((user_attribute_id, policy_class_id)) => create_association_cypher(
            NodeType::UserAttribute,
            user_attribute_id,
            NodeType::PolicyClass,
            policy_class_id,
            AHashMap::new(),
        ),
        Assignment::O2OA((object_id, object_attribute_id)) => create_association_cypher(
            NodeType::Object,
            object_id,
            NodeType::ObjectAttribute,
            object_attribute_id,
            AHashMap::new(),
        ),
        Assignment::OA2OA((sub_object_attribute_id, object_attribute_id)) => {
            create_association_cypher(
                NodeType::ObjectAttribute,
                sub_object_attribute_id,
                NodeType::ObjectAttribute,
                object_attribute_id,
                AHashMap::new(),
            )
        }
        Assignment::OA2PC((object_attribute_id, policy_class_id)) => create_association_cypher(
            NodeType::ObjectAttribute,
            object_attribute_id,
            NodeType::PolicyClass,
            policy_class_id,
            AHashMap::new(),
        ),
    };

    let st = match age.prepare_cypher(GRAPH_NAME, &cypher, false).await {
        Ok(st) => st,
        Err(e) => return Some(Status::from_error(Box::new(e))),
    };
    if let Err(e) = age.query(&st, &[]).await {
        return Some(Status::from_error(Box::new(e)));
    }
    None
}

pub fn search_node_cypher(
    node_type: NodeType,
    name: Option<&str>,
    id: Option<i64>,
    properties: AHashMap<FastStr, FastStr>,
) -> Result<String, Status> {
    match (name, id) {
        (Some(name), Some(id)) => Ok(format!(
            "{} ({}: {} {{ name: '{}' }}) {} {} = {}{} {} {}",
            MATCH,
            node_type,
            node_type.fmt_full(),
            name.to_string(),
            WHERE,
            OpenCypherFunc::id("u"),
            id,
            properties_to_query_condition(&node_type, properties),
            RETURN,
            node_type
        )),
        (Some(name), None) => Ok(format!(
            "{} ({}: {}) {} {}.name = '{}'{} {} {}",
            MATCH,
            node_type,
            node_type.fmt_full(),
            WHERE,
            node_type,
            name,
            properties_to_query_condition(&node_type, properties),
            RETURN,
            node_type
        )),
        (None, Some(id)) => Ok(format!(
            "{} ({}: {}) {} {} = {}{} {} {}",
            MATCH,
            node_type,
            node_type.fmt_full(),
            WHERE,
            OpenCypherFunc::id("u"),
            id,
            properties_to_query_condition(&node_type, properties),
            RETURN,
            node_type
        )),
        (None, None) => Err(Status::invalid_argument(
            "The name and ID cannot both be empty!",
        )),
    }
}

pub fn search_origin_id_to_assigned_target_node_path_cypher(
    origin_node_type: NodeType,
    origin_id: i64,
    target_node_type: NodeType,
    target_node_name: Option<&str>,
    target_node_id: Option<i64>,
    target_node_properties: AHashMap<FastStr, FastStr>,
    adjacent: bool,
) -> Result<String, Status> {
    match (target_node_name, target_node_id) {
        (None, None) => Err(Status::invalid_argument(
            "The name and ID cannot both be empty!",
        )),
        (None, Some(id)) => match properties_to_property_query_condition(target_node_properties) {
            Some(properties) => Ok(format!(
                "{} ({}: {})-[{}]->({}: {} {{{}}}) {} {} = {} {} {} = {} {} {}",
                MATCH,
                origin_node_type,
                origin_node_type.fmt_full(),
                if adjacent { "" } else { "*" },
                target_node_type,
                target_node_type.fmt_full(),
                properties,
                WHERE,
                OpenCypherFunc::id(&origin_node_type.to_string()),
                origin_id,
                AND,
                OpenCypherFunc::id(&target_node_type.to_string()),
                id,
                RETURN,
                target_node_type
            )),
            None => Ok(format!(
                "{} ({}: {})-[{}]->({}: {}) {} {} = {} {} {} = {} {} {}",
                MATCH,
                origin_node_type,
                origin_node_type.fmt_full(),
                if adjacent { "" } else { "*" },
                target_node_type,
                target_node_type.fmt_full(),
                WHERE,
                OpenCypherFunc::id(&origin_node_type.to_string()),
                origin_id,
                AND,
                OpenCypherFunc::id(&target_node_type.to_string()),
                id,
                RETURN,
                target_node_type
            )),
        },
        (Some(name), None) => {
            match properties_to_property_query_condition(target_node_properties) {
                Some(properties) => Ok(format!(
                    "{} ({}: {})-[{}]->({}: {} {{ name: '{}',{}}}) {} {} = {} {} {}",
                    MATCH,
                    origin_node_type,
                    origin_node_type.fmt_full(),
                    if adjacent { "" } else { "*" },
                    target_node_type,
                    target_node_type.fmt_full(),
                    name,
                    properties,
                    WHERE,
                    OpenCypherFunc::id(&origin_node_type.to_string()),
                    origin_id,
                    RETURN,
                    target_node_type
                )),
                None => Ok(format!(
                    "{} ({}: {})-[{}]->({}: {} {{ name: '{}' }}) {} {} = {} {} {}",
                    MATCH,
                    origin_node_type,
                    origin_node_type.fmt_full(),
                    if adjacent { "" } else { "*" },
                    target_node_type,
                    target_node_type.fmt_full(),
                    name,
                    WHERE,
                    OpenCypherFunc::id(&origin_node_type.to_string()),
                    origin_id,
                    RETURN,
                    target_node_type
                )),
            }
        }
        (Some(name), Some(id)) => {
            match properties_to_property_query_condition(target_node_properties) {
                Some(properties) => Ok(format!(
                    "{} ({}: {})-[{}]->({}: {} {{name: '{}'{}}}) {} {} = {} {} {} = {} {} {}",
                    MATCH,
                    origin_node_type,
                    origin_node_type.fmt_full(),
                    if adjacent { "" } else { "*" },
                    target_node_type,
                    target_node_type.fmt_full(),
                    name,
                    properties,
                    WHERE,
                    OpenCypherFunc::id(&origin_node_type.to_string()),
                    origin_id,
                    AND,
                    OpenCypherFunc::id(&target_node_type.to_string()),
                    id,
                    RETURN,
                    target_node_type
                )),
                None => Ok(format!(
                    "{} ({}: {})-[{}]->({}: {} {{ name: '{}' }}) {} {} = {} {} {}",
                    MATCH,
                    origin_node_type,
                    origin_node_type.fmt_full(),
                    if adjacent { "" } else { "*" },
                    target_node_type,
                    target_node_type.fmt_full(),
                    name,
                    WHERE,
                    OpenCypherFunc::id(&origin_node_type.to_string()),
                    origin_id,
                    RETURN,
                    target_node_type
                )),
            }
        }
    }
}

pub async fn search_node(
    node_type: NodeType,
    name: Option<&str>,
    id: Option<i64>,
    properties: AHashMap<FastStr, FastStr>,
) -> Result<VertexTypeObject, Status> {
    let age = match get_age().await {
        Ok(age) => age,
        Err(e) => return Err(Status::from_error(e)),
    };

    let cypher = match search_node_cypher(node_type.clone(), name, id, properties) {
        Ok(cypher) => cypher,
        Err(s) => return Err(s),
    };

    match age.query_cypher::<()>(GRAPH_NAME, &cypher, None).await {
        Ok(rows) => {
            if !rows.is_empty() {
                match node_type {
                    NodeType::User => {
                        let user: Vertex<User> = rows[0].get(0);
                        Ok(VertexTypeObject::User(user))
                    }
                    NodeType::UserAttribute => {
                        let user_attribute: Vertex<UserAttribute> = rows[0].get(0);
                        Ok(VertexTypeObject::UserAttribute(user_attribute))
                    }
                    NodeType::Object => {
                        let object: Vertex<Object> = rows[0].get(0);
                        Ok(VertexTypeObject::Object(object))
                    }
                    NodeType::ObjectAttribute => {
                        let object_attribute: Vertex<ObjectAttribute> = rows[0].get(0);
                        Ok(VertexTypeObject::ObjectAttribute(object_attribute))
                    }
                    NodeType::PolicyClass => {
                        let policy_calass: Vertex<PolicyClass> = rows[0].get(0);
                        Ok(VertexTypeObject::PolicyClass(policy_calass))
                    }
                }
            } else {
                Err(Status::not_found("node not found!"))
            }
        }
        Err(e) => Err(Status::from_error(Box::new(e))),
    }
}

pub async fn search_user_attribute_node(
    id: Option<i64>,
    attribute_name: Option<&str>,
    properties: AHashMap<FastStr, FastStr>,
) -> Result<Vertex<UserAttribute>, Status> {
    let age = match get_age().await {
        Ok(age) => age,
        Err(e) => return Err(Status::from_error(e)),
    };

    let cypher = match search_node_cypher(NodeType::UserAttribute, attribute_name, id, properties) {
        Ok(cypher) => cypher,
        Err(s) => return Err(s),
    };

    let user_attribute = match age.query_cypher::<()>(GRAPH_NAME, &cypher, None).await {
        Ok(rows) => {
            if !rows.is_empty() {
                let node: Vertex<UserAttribute> = rows[0].get(0);
                node
            } else {
                return Err(Status::not_found("User attribute not found!"));
            }
        }
        Err(e) => return Err(Status::from_error(Box::new(e))),
    };
    Ok(user_attribute)
}

// pub async fn search_user_attribute_node_with_assigned_id(
//     assigned_id: i64,
//     assigned_node_type: UserAttributeOriginNodeType,
//     attribute_name: Option<&str>,
//     id: Option<i64>,
//     properties: AHashMap<FastStr, FastStr>,
//     adjacent: bool,
// ) -> Result<Vertex<UserAttribute>, Status> {
//     let age = match get_age().await {
//         Ok(age) => age,
//         Err(e) => return Err(Status::from_error(e)),
//     };
//     let mut target_node_type = NodeType::UserAttribute;
//     if assigned_node_type == UserAttributeOriginNodeType::USER {
//         target_node_type = NodeType::User;
//     }
//     let cypher = match search_origin_id_to_assigned_target_node_path_cypher(
//         target_node_type,
//         assigned_id,
//         NodeType::UserAttribute,
//         attribute_name,
//         id,
//         properties,
//         adjacent,
//     ) {
//         Ok(cypher) => cypher,
//         Err(s) => return Err(s),
//     };

//     let user_attribute = match age.query_cypher::<()>(GRAPH_NAME, &cypher, None).await {
//         Ok(rows) => {
//             if !rows.is_empty() {
//                 let node: Vertex<UserAttribute> = rows[0].get(0);
//                 node
//             } else {
//                 return Err(Status::not_found("User attribute not found!"));
//             }
//         }
//         Err(e) => return Err(Status::from_error(Box::new(e))),
//     };
//     Ok(user_attribute)
// }