use serde_json::Value;

#[derive(Debug)]
pub enum Operator {
    CREATE,
    MATCH,
    DELETE,
    UPDATE,
}

#[derive(Debug)]
pub enum AuxiliaryOperator {
    RETURN,
    LIMIT,
    WHERE,
    SET,
}

#[derive(Debug, Clone)]
pub enum PermissionOperator {
    Add,
    Edit,
    Read,
    Delete,
}

#[derive(Debug)]
pub enum NodeType {
    USER,
    USER_ATTRIBUTES,
    OBJECT,
    OBJECT_ATTRIBUTES,
    POLICY_CLASS,
    ASSIGNMENT,
    ASSOCIATION,
    PROHIBITION,
}

#[derive(Debug)]
pub enum Property {
    User(UserProperties),
}

#[derive(Debug)]
pub struct UserProperties {
    pub name: String,
}

impl std::fmt::Display for UserProperties {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let begin_of_struct = " ".repeat(2);
        let end_of_struct = " ".repeat(2);
        let begin_of_property = " ".repeat(4);
        let between_key_values = " ";
        write!(f, "{}{{\n{}{}:{}'{}'\n{}}}", begin_of_struct, begin_of_property, "name", between_key_values, self.name, end_of_struct)
    }
}

impl UserProperties {
    fn set_fmt(&self) -> String {
        format!("{} = \"{}\"", "name", self.name)
    }
}

pub fn format_cql(categovy: Operator, node_type: NodeType, property: Option<Property>, where_str: Option<String>) -> String {
    match categovy {
        Operator::CREATE => {
            match property {
                Some(pro) => {
                    match pro {
                        Property::User(p) => format!("{:#?} (\n  n:{:#?}\n{}\n)", categovy, node_type, p.to_string()),
                    }
                },
                None => format!("{:#?} (\n  n:{:#?}\n)", categovy, node_type),
            }
        },
        Operator::MATCH => format!("{:#?} (n:{:#?}) {:#?} n {:#?} 100", categovy, node_type, AuxiliaryOperator::RETURN, AuxiliaryOperator::LIMIT),
        Operator::DELETE => {
            match property {
                Some(pro) => {
                    match pro {
                        Property::User(p) => {
                            match where_str {
                                Some(w) => format!("{:#?} (\n  n:{:#?}\n{}\n) {} {:#?} n", Operator::MATCH, node_type, p.to_string(), w, categovy),
                                None => format!("{:#?} (\n  n:{:#?}\n{}\n) {:#?} n", Operator::MATCH, node_type, p.to_string(), categovy),
                            }
                        },
                    }
                },
                None => format!("{:#?} (n:{:#?}) {:#?} {:#?}", categovy, node_type, categovy, node_type),
            }
        },
        Operator::UPDATE => {
            match property {
                Some(pro) => {
                    match pro {
                        Property::User(p) => {
                            match where_str {
                                Some(w) => format!("{:#?} (\n  n:{:#?}\n) {} {:#?} {:#?}.{} {:#?} n", Operator::MATCH, node_type, w, AuxiliaryOperator::SET, node_type, p.set_fmt(), AuxiliaryOperator::RETURN),
                                None => format!("{:#?} (\n  n:{:#?}\n) {:#?} {:#?}.{} {:#?} n", Operator::MATCH, node_type, AuxiliaryOperator::SET, node_type, p.set_fmt(), AuxiliaryOperator::RETURN),
                            }
                        },
                    }
                },
                None => todo!(),
            }
        }
    }
}

pub fn format_where(node: NodeType, keys: Vec<&str>, values: Vec<Value>, operators: Vec<&str>) -> String {
    let mut where_str = format!("{:#?} ", AuxiliaryOperator::WHERE);
    for (index, item) in keys.iter().enumerate() {
        where_str = where_str + &format!("\n{:#?}.{} {} {}", node, item, operators[index], values[index]);
    }
    where_str = where_str + "\n";
    where_str
}

pub fn format_id_where(values: Vec<i64>) -> String {
    let mut where_str = format!("{:#?} ", AuxiliaryOperator::WHERE);
    for item in values.iter() {
        where_str = where_str + &format!("\nID(n) = {}", item);
    }
    where_str = where_str + "\n";
    where_str
}

pub fn format_path_query(user_id: i64, object_id: i64, ops: PermissionOperator) -> String {
    format!(
        "MATCH path = (u:{:#?})-[*]->(ua:{:#?})-[r:{:#?}]->(oa:{:#?})<-[*]-(o:{:#?}) WHERE ID(u) = {} AND ID(o) = {} AND '{:#?}' IN r.ops MATCH (ua)-[:{:#?}]->(pc1:{:#?}) MATCH (oa)-[:{:#?}]->(pc2:{:#?}) where pc1.name = pc2.name RETURN path, pc1, o, oa, u, ua, r ORDER BY length(path) LIMIT 1",
        NodeType::USER,
        NodeType::USER_ATTRIBUTES,
        NodeType::ASSOCIATION,
        NodeType::OBJECT_ATTRIBUTES,
        NodeType::OBJECT,
        user_id,
        object_id,
        ops,
        NodeType::ASSIGNMENT,
        NodeType::POLICY_CLASS,
        NodeType::ASSIGNMENT,
        NodeType::POLICY_CLASS,
    )
}
