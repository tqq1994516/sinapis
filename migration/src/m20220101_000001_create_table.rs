use sea_orm_migration::{prelude::*, schema::*, sea_orm::IntoSimpleExpr, sea_query::extension::postgres::Type};

use super::utils::{SnowflakeId, UserProperty, Gender};

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
        manager
            .create_table(
                Table::create()
                    .table(UserProperty::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserProperty::Id)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(string(UserProperty::Name))
                    .col(blob(UserProperty::Password))
                    .col(string_null(UserProperty::Alias))
                    .col(string_null(UserProperty::Email))
                    .col(string_null(UserProperty::Phone))
                    .col(json_binary_null(UserProperty::Extra))
                    .col(
                        ColumnDef::new(UserProperty::CreatedAt)
                            .date_time()
                            .null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(UserProperty::UpdatedAt)
                            .date_time()
                            .null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // 添加更新 `updated_at` 字段的函数
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE OR REPLACE FUNCTION update_updated_at_column()
                RETURNS TRIGGER AS $$
                BEGIN
                    NEW.updated_at = NOW();
                    RETURN NEW;
                END;
                $$ language 'plpgsql';
                "#,
            )
            .await?;

        // 添加触发器，每次更新行时更新 `updated_at`
        manager
            .get_connection()
            .execute_unprepared(&format!(
                r#"
                CREATE TRIGGER update_{table}_updated_at
                BEFORE UPDATE ON {table}
                FOR EACH ROW
                EXECUTE PROCEDURE update_updated_at_column();
                "#,
                table = UserProperty::Table.to_string()
            ))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 删除触发器
        manager
            .get_connection()
            .execute_unprepared(&format!(
                r#"
                DROP TRIGGER IF EXISTS update_{table}_updated_at ON {table};
                "#,
                table = UserProperty::Table.to_string()
            ))
            .await?;

        // 删除函数
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                DROP FUNCTION IF EXISTS update_updated_at_column;
                "#,
            )
            .await?;

        manager
            .drop_table(Table::drop().table(UserProperty::Table).to_owned())
            .await
    }
}
