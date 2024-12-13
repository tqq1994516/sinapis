use volo_grpc::{Status, Request, Response};
use sea_orm::DatabaseConnection;
use bb8::Pool;
use bb8_postgres::{tokio_postgres::GenericClient, PostgresConnectionManager};

use volo_gen::person_center::{
    UserAttribute,
    AddUserAttributeRequest,
    EditUserAttributeRequest,
    FilterAttributeRequest,
    PreciseAttributeRequest,
    UserAttributeResponse,
    UserAttributesResponse,
    Accessable,
};
use pool::age::{AgeClientExtend, Client, NoTls};

use crate::service::user_attribute::{
    handler_add_user_attribute,
    handler_search_user_attribute,
};

#[derive(Debug, Default)]
pub struct UserAttributeService;

impl UserAttribute for UserAttributeService {
    async fn add_user_attribute(&self, req: Request<AddUserAttributeRequest>) -> Result<Response<UserAttributeResponse>, Status> {
        let (_, extensions, data) = req.into_parts();
        let db = extensions.get::<DatabaseConnection>().ok_or_else(|| Status::aborted("pg orm connection not found"))?;
        let pg_pool = extensions.get::<Pool<PostgresConnectionManager<NoTls>>>().ok_or_else(|| Status::aborted("pg connection not found"))?;
        let pg_connect = pg_pool.get().await.map_err(|e| Status::aborted("pg connection not found"))?;
        let pg_client = pg_connect.client();
        let age_client = Client::connect_age_extend(pg_client).await.map_err(|e| Status::aborted("pg connection not found"))?;
        handler_add_user_attribute(data, db, age_client).await
    }

    async fn edit_user_attribute(&self, req: Request<EditUserAttributeRequest>) -> Result<Response<UserAttributeResponse>, Status> {
        todo!()
    }

    async fn filter_user_attribute(&self, req: Request<FilterAttributeRequest>) -> Result<Response<UserAttributesResponse>, Status> {
        let (_, extensions, data) = req.into_parts();
        let pg_pool = extensions.get::<Pool<PostgresConnectionManager<NoTls>>>().ok_or_else(|| Status::aborted("pg connection not found"))?;
        let pg_connect = pg_pool.get().await.map_err(|e| Status::aborted("pg connection not found"))?;
        let pg_client = pg_connect.client();
        let age_client = Client::connect_age_extend(pg_client).await.map_err(|e| Status::aborted("pg connection not found"))?;
        handler_search_user_attribute(data, age_client).await
    }

    async fn remove_user_attribute(&self, req: Request<PreciseAttributeRequest>) -> Result<Response<Accessable>, Status> {
        todo!()
    }
}
