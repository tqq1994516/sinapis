use sea_orm_migration::{prelude::*, sea_orm::IntoSimpleExpr, sea_query::extension::postgres::Type};

use super::utils::{SnowflakeId, UserInfo, Gender};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(Gender::Enum)
                    .values([Gender::Male, Gender::Female])
                    .to_owned(),
            )
            .await?;

        // 雪花漂移id
        // manager
        //     .create_table(
        //         Table::create()
        //             .table(UserInfo::Table)
        //             .if_not_exists()
        //             .col(
        //                 ColumnDef::new(UserInfo::Id)
        //                     .big_integer()
        //                     .not_null()
        //                     .primary_key()
        //                     .default(Expr::expr(Func::cust(SnowflakeId).args([Expr::expr(Func::random()).into_simple_expr()]))),
        //             )
        //             .col(ColumnDef::new(UserInfo::Username).string().not_null())
        //             .col(ColumnDef::new(UserInfo::Password).string().not_null())
        //             .col(ColumnDef::new(UserInfo::FirstName).string())
        //             .col(ColumnDef::new(UserInfo::LastName).string())
        //             .col(ColumnDef::new(UserInfo::Birthday).date())
        //             .col(ColumnDef::new(UserInfo::Gender).custom(Gender::Enum))
        //             .col(ColumnDef::new(UserInfo::Email).string())
        //             .col(ColumnDef::new(UserInfo::Phone).string())
        //             .col(ColumnDef::new(UserInfo::CreateTime).timestamp_with_time_zone().default(Expr::current_timestamp()))
        //             .col(ColumnDef::new(UserInfo::UpdateTime).timestamp_with_time_zone().default(Expr::current_timestamp()))
        //             .col(ColumnDef::new(UserInfo::LatestLoginTime).timestamp_with_time_zone())
        //             .col(ColumnDef::new(UserInfo::Online).boolean().default(false))
        //             .col(ColumnDef::new(UserInfo::Neo4jId).big_integer())
        //             .index(Index::create().unique().name("idx-neo4j-id").col(UserInfo::Neo4jId))
        //             .col(ColumnDef::new(UserInfo::Extra).json_binary())
        //             .to_owned(),
        //     )
        //     .await?;

        // 自增id
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
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(UserInfo::Username).string().not_null())
                    .col(ColumnDef::new(UserInfo::Password).string().not_null())
                    .col(ColumnDef::new(UserInfo::FirstName).string())
                    .col(ColumnDef::new(UserInfo::LastName).string())
                    .col(ColumnDef::new(UserInfo::Birthday).date())
                    .col(ColumnDef::new(UserInfo::Gender).custom(Gender::Enum))
                    .col(ColumnDef::new(UserInfo::Email).string())
                    .col(ColumnDef::new(UserInfo::Phone).string())
                    .col(ColumnDef::new(UserInfo::CreateTime).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(UserInfo::UpdateTime).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(UserInfo::LatestLoginTime).timestamp_with_time_zone())
                    .col(ColumnDef::new(UserInfo::Online).boolean().default(false))
                    .col(ColumnDef::new(UserInfo::Neo4jId).big_integer())
                    .index(Index::create().unique().name("idx-neo4j-id").col(UserInfo::Neo4jId))
                    .col(ColumnDef::new(UserInfo::Extra).json_binary())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        let _ = manager
            .drop_table(Table::drop().table(UserInfo::Table).to_owned())
            .await;

        let _ = manager
            .drop_type(Type::drop().name(Gender::Enum).to_owned())
            .await;

        Ok(())
    }
}
