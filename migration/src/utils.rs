use sea_orm_migration::prelude::*;

pub struct SnowflakeId;

impl Iden for SnowflakeId {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(s, "fn_next_snow_id").unwrap();
    }
}

#[derive(DeriveIden)]
pub enum UserInfo {
    Table,
    Id,
    Username,
    Password,
    FirstName,
    LastName,
    Birthday,
    Gender,
    Email,
    Phone,
    CreateTime,
    UpdateTime,
    LatestLoginTime,
    Online,
    Neo4jId,
    Extra,
}

#[derive(DeriveIden)]
pub enum Gender {
    #[sea_orm(iden = "gender")]
    Enum,
    #[sea_orm(iden = "female")]
    Female,
    #[sea_orm(iden = "male")]
    Male,
}
