use sea_orm_migration::prelude::*;

pub struct SnowflakeId;

impl Iden for SnowflakeId {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(s, "fn_next_snow_id").unwrap();
    }
}

#[derive(DeriveIden)]
pub enum UserProperty {
    Table,
    Id,
    Name,
    Password,
    Alias,
    Email,
    Phone,
    Extra,
    CreatedAt,
    UpdatedAt,
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
