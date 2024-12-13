use volo_grpc::{Status, Request, Response};
use sea_orm::DatabaseConnection;
use bb8::Pool;
use bb8_postgres::{tokio_postgres::GenericClient, PostgresConnectionManager};

use volo_gen::person_center::{
    User,
    UsersResponse,
    UserResponse,
    LoginForm,
    Logged,
    Accessable,
    FilterUserRequest,
    UserDetailRequest,
    EditUserRequest,
    PrivateUserInfo,
    CheckPermissionRequest,
};
use pool::age::{AgeClientExtend, Client, NoTls};

use crate::service::user::{
    handler_add_user, handler_login, handler_search_user
};

#[derive(Debug, Default)]
pub struct UserService;

impl User for UserService {
	async fn user_list(&self, req: Request<FilterUserRequest>) -> Result<Response<UsersResponse>, Status> {
        let (_, extensions, data) = req.into_parts();
        let db = extensions.get::<DatabaseConnection>().ok_or_else(|| Status::aborted("pg orm connection not found"))?;
        let pg_pool = extensions.get::<Pool<PostgresConnectionManager<NoTls>>>().ok_or_else(|| Status::aborted("pg connection not found"))?;
        let pg_connect = pg_pool.get().await.map_err(|_| Status::aborted("pg connection not found"))?;
        let pg_client = pg_connect.client();
        let age_client = Client::connect_age_extend(pg_client).await.map_err(|_| Status::aborted("pg connection not found"))?;
        handler_search_user(data, db, age_client).await
    }

	async fn user_detail(&self, req: Request<UserDetailRequest>) -> Result<Response<UserResponse>, Status> {
        todo!()
    }

	async fn update_user(&self, req: Request<EditUserRequest>) -> Result<Response<UserResponse>, Status> {
        todo!()
    }

	async fn insert_user(&self, req: Request<PrivateUserInfo>) -> Result<Response<UserResponse>, Status> {
        let (_, extensions, data) = req.into_parts();
        let db = extensions.get::<DatabaseConnection>().ok_or_else(|| Status::aborted("pg orm connection not found"))?;
        let pg_pool = extensions.get::<Pool<PostgresConnectionManager<NoTls>>>().ok_or_else(|| Status::aborted("pg connection not found"))?;
        let pg_connect = pg_pool.get().await.map_err(|e| Status::aborted("pg connection not found"))?;
        let pg_client = pg_connect.client();
        let age_client = Client::connect_age_extend(pg_client).await.map_err(|e| Status::aborted("pg connection not found"))?;
        handler_add_user(data, db, age_client).await
    }

	async fn delete_user(&self, req: Request<UserDetailRequest>) -> Result<Response<Accessable>, Status> {
        todo!()
    }

	async fn login(&self, req: Request<LoginForm>) -> Result<Response<Logged>, Status> {
        let (_, extensions, data) = req.into_parts();
        let db = extensions.get::<DatabaseConnection>().ok_or_else(|| Status::aborted("pg orm connection not found"))?;
        let pg_pool = extensions.get::<Pool<PostgresConnectionManager<NoTls>>>().ok_or_else(|| Status::aborted("pg connection not found"))?;
        let pg_connect = pg_pool.get().await.map_err(|e| Status::aborted("pg connection not found"))?;
        let pg_client = pg_connect.client();
        let age_client = Client::connect_age_extend(pg_client).await.map_err(|e| Status::aborted("pg connection not found"))?;
        handler_login(data, db, age_client).await
    }

	async fn check_permission(&self, req: Request<CheckPermissionRequest>) -> Result<Response<Accessable>, Status> {
        todo!()
    }
}
