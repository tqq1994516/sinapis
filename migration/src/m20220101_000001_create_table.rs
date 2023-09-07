use sea_orm_migration::{prelude::*, sea_orm::IntoSimpleExpr};

use super::utils::{SnowflakeId, UserInfo, Organization};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .create_table(
                Table::create()
                    .table(Organization::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Organization::Id)
                            .big_integer()
                            .not_null()
                            .primary_key()
                            .default(Expr::expr(Func::cust(SnowflakeId).args([Expr::expr(Func::random()).into_simple_expr()]))),
                    )
                    .col(ColumnDef::new(Organization::Name).string().not_null())
                    .col(ColumnDef::new(Organization::CreateTime).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Organization::UpdateTime).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Organization::Available).boolean().default(true))
                    .col(ColumnDef::new(Organization::Accessible).boolean().default(true))
                    .col(ColumnDef::new(Organization::PeriodOfValidity).timestamp_with_time_zone().null())
                    .to_owned(),
            ).await?;

        manager
            .create_table(
                Table::create()
                    .table(UserInfo::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserInfo::Id)
                            .big_integer()
                            .not_null()
                            .primary_key()
                            .default(Expr::expr(Func::cust(SnowflakeId).args([Expr::expr(Func::random()).into_simple_expr()]))),
                    )
                    .col(ColumnDef::new(UserInfo::Name).string().not_null())
                    .col(ColumnDef::new(UserInfo::Password).string().not_null())
                    .col(ColumnDef::new(UserInfo::Email).string().unique_key().null())
                    .col(ColumnDef::new(UserInfo::Phone).string().unique_key().null())
                    .col(ColumnDef::new(UserInfo::Online).boolean().default(false))
                    .col(ColumnDef::new(UserInfo::Info).json().null())
                    .col(ColumnDef::new(UserInfo::CreateTime).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(UserInfo::UpdateTime).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(UserInfo::Available).boolean().default(true))
                    .col(ColumnDef::new(UserInfo::Accessible).boolean().default(true))
                    .col(ColumnDef::new(UserInfo::PeriodOfValidity).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(UserInfo::Organization).big_integer().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-user_info-organization_id")
                            .from(UserInfo::Table, UserInfo::Organization)
                            .to(Organization::Table, Organization::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        
        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name("idx-user_info-email")
                    .table(UserInfo::Table)
                    .col(UserInfo::Email)
                    .to_owned()
            )
            .await?;

        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name("idx-user_info-phone")
                    .table(UserInfo::Table)
                    .col(UserInfo::Phone)
                    .to_owned()
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .drop_table(Table::drop().table(UserInfo::Table).to_owned())
            .await
    }
}
