use sea_orm_migration::prelude::*;

pub struct SnowflakeId;

impl Iden for SnowflakeId {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(s, "fn_next_snow_id").unwrap();
    }
}

#[derive(DeriveIden)]
pub enum Organization {
    Table,
    Id,
    Name,
    CreateTime,
    UpdateTime,
    Available,
    Accessible,
    PeriodOfValidity,
}

#[derive(DeriveIden)]
pub enum UserInfo {
    Table,
    Id,
    Name,
    Password,
    Email,
    Phone,
    Online,
    Info,
    CreateTime,
    UpdateTime,
    Available,
    Organization,
    Accessible,
    PeriodOfValidity,
}

#[derive(DeriveIden)]
pub enum Role {
    Table,
    Id,
    Name,
    CreateTime,
    UpdateTime,
    Owner,
    Available,
    Organization,
}

#[derive(DeriveIden)]
pub enum Permission {
    Table,
    Id,
    Name,
    CreateTime,
    UpdateTime,
    Owner,
    Available,
    Organization,
}

#[derive(DeriveIden)]
pub enum Policy {
    Table,
    Id,
    Name,
    ResourceType,
    Resource,
    Action,
    Allow,
    CreateTime,
    UpdateTime,
    Owner,
    Available,
    Organization,
}

#[derive(DeriveIden)]
pub enum ResourceType {
    Table,
    Id,
    Name,
    CreateTime,
    UpdateTime,
    Owner,
    Available,
    Organization,
}

#[derive(DeriveIden)]
pub enum UserRole {
    Table,
    Id,
    User,
    Role,
}

#[derive(DeriveIden)]
pub enum RoleParent {
    Table,
    Id,
    Child,
    Parent,
}

#[derive(DeriveIden)]
pub enum RolePermission {
    Table,
    Id,
    Role,
    Permission,
}

#[derive(DeriveIden)]
pub enum RolePolicy {
    Table,
    Id,
    Role,
    Policy,
}
