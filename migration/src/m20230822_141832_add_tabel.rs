use sea_orm_migration::{prelude::*, sea_orm::IntoSimpleExpr};

use super::utils::{
    SnowflakeId,
    UserInfo,
    UserRole,
    Role,
    Permission,
    Policy,
    ResourceType,
    RoleParent,
    RolePermission,
    RolePolicy,
    Organization,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .create_table(
                Table::create()
                    .table(Role::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Role::Id)
                            .big_integer()
                            .not_null()
                            .primary_key()
                            .default(Expr::expr(Func::cust(SnowflakeId).args([Expr::expr(Func::random()).into_simple_expr()]))),
                    )
                    .col(ColumnDef::new(Role::Name).string().not_null())
                    .col(ColumnDef::new(Role::CreateTime).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Role::UpdateTime).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Role::Owner).big_integer().null())
                    .col(ColumnDef::new(Role::Available).boolean().default(true))
                    .col(ColumnDef::new(Role::Organization).big_integer().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-role-user_info_id")
                            .from(Role::Table, Role::Owner)
                            .to(UserInfo::Table, UserInfo::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-role-organization_id")
                            .from(Role::Table, Role::Organization)
                            .to(Organization::Table, Organization::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Permission::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Permission::Id)
                            .big_integer()
                            .not_null()
                            .primary_key()
                            .default(Expr::expr(Func::cust(SnowflakeId).args([Expr::expr(Func::random()).into_simple_expr()]))),
                    )
                    .col(ColumnDef::new(Permission::Name).string().not_null())
                    .col(ColumnDef::new(Permission::CreateTime).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Permission::UpdateTime).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Permission::Owner).big_integer().null())
                    .col(ColumnDef::new(Permission::Available).boolean().default(true))
                    .col(ColumnDef::new(Permission::Organization).big_integer().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-permission-user_info_id")
                            .from(Permission::Table, Permission::Owner)
                            .to(UserInfo::Table, UserInfo::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-permission-organization_id")
                            .from(Permission::Table, Permission::Organization)
                            .to(Organization::Table, Organization::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ResourceType::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ResourceType::Id)
                            .big_integer()
                            .not_null()
                            .primary_key()
                            .default(Expr::expr(Func::cust(SnowflakeId).args([Expr::expr(Func::random()).into_simple_expr()]))),
                    )
                    .col(ColumnDef::new(ResourceType::Name).string().not_null())
                    .col(ColumnDef::new(ResourceType::CreateTime).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(ResourceType::UpdateTime).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(ResourceType::Owner).big_integer().null())
                    .col(ColumnDef::new(ResourceType::Available).boolean().default(true))
                    .col(ColumnDef::new(ResourceType::Organization).big_integer().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-resource_type-user_info_id")
                            .from(ResourceType::Table, ResourceType::Owner)
                            .to(UserInfo::Table, UserInfo::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-resource_type-organization_id")
                            .from(ResourceType::Table, ResourceType::Organization)
                            .to(Organization::Table, Organization::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Policy::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Policy::Id)
                            .big_integer()
                            .not_null()
                            .primary_key()
                            .default(Expr::expr(Func::cust(SnowflakeId).args([Expr::expr(Func::random()).into_simple_expr()]))),
                    )
                    .col(ColumnDef::new(Policy::Name).string().not_null())
                    .col(ColumnDef::new(Policy::ResourceType).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-policy-resource_type_id")
                            .from(Policy::Table, Policy::ResourceType)
                            .to(ResourceType::Table, ResourceType::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Policy::Resource).big_integer().not_null())
                    .col(ColumnDef::new(Policy::Action).string().not_null())
                    .col(ColumnDef::new(Policy::Allow).string().not_null())
                    .col(ColumnDef::new(Policy::CreateTime).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Policy::UpdateTime).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Policy::Owner).big_integer().null())
                    .col(ColumnDef::new(Policy::Available).boolean().default(true))
                    .col(ColumnDef::new(Policy::Organization).big_integer().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-policy-user_info_id")
                            .from(Policy::Table, Policy::Owner)
                            .to(UserInfo::Table, UserInfo::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-policy-organization_id")
                            .from(Policy::Table, Policy::Organization)
                            .to(Organization::Table, Organization::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(UserRole::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserRole::Id)
                            .big_integer()
                            .not_null()
                            .primary_key()
                            .default(Expr::expr(Func::cust(SnowflakeId).args([Expr::expr(Func::random()).into_simple_expr()]))),
                    )
                    .col(ColumnDef::new(UserRole::User).big_integer().null())
                    .col(ColumnDef::new(UserRole::Role).big_integer().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-user_role-user_id")
                            .from(UserRole::Table, UserRole::User)
                            .to(UserInfo::Table, UserInfo::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-user_role-role_id")
                            .from(UserRole::Table, UserRole::Role)
                            .to(Role::Table, Role::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(RoleParent::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RoleParent::Id)
                            .big_integer()
                            .not_null()
                            .primary_key()
                            .default(Expr::expr(Func::cust(SnowflakeId).args([Expr::expr(Func::random()).into_simple_expr()]))),
                    )
                    .col(ColumnDef::new(RoleParent::Child).big_integer().null())
                    .col(ColumnDef::new(RoleParent::Parent).big_integer().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-role_parent-child_id")
                            .from(RoleParent::Table, RoleParent::Child)
                            .to(Role::Table, Role::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-role_parent-parent_id")
                            .from(RoleParent::Table, RoleParent::Parent)
                            .to(Role::Table, Role::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(RolePermission::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RolePermission::Id)
                            .big_integer()
                            .not_null()
                            .primary_key()
                            .default(Expr::expr(Func::cust(SnowflakeId).args([Expr::expr(Func::random()).into_simple_expr()]))),
                    )
                    .col(ColumnDef::new(RolePermission::Role).big_integer().null())
                    .col(ColumnDef::new(RolePermission::Permission).big_integer().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-role_permission-role_id")
                            .from(RolePermission::Table, RolePermission::Role)
                            .to(Role::Table, Role::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-role_permission-permission_id")
                            .from(RolePermission::Table, RolePermission::Permission)
                            .to(Permission::Table, Permission::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(RolePolicy::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RolePolicy::Id)
                            .big_integer()
                            .not_null()
                            .primary_key()
                            .default(Expr::expr(Func::cust(SnowflakeId).args([Expr::expr(Func::random()).into_simple_expr()]))),
                    )
                    .col(ColumnDef::new(RolePolicy::Role).big_integer().null())
                    .col(ColumnDef::new(RolePolicy::Policy).big_integer().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-role_policy-role_id")
                            .from(RolePolicy::Table, RolePolicy::Role)
                            .to(Role::Table, Role::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-role_policy-policy_id")
                            .from(RolePolicy::Table, RolePolicy::Policy)
                            .to(Policy::Table, Policy::Id)
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
                    .name("idx-policy-resource")
                    .table(Policy::Table)
                    .col(Policy::Resource)
                    .to_owned()
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Role::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Permission::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Policy::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ResourceType::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(UserRole::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(RoleParent::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(RolePermission::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(RolePolicy::Table).to_owned())
            .await?;

        Ok(())
    }
}
