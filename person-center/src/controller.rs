use neo4rs::Graph;
use volo_grpc::{Status, Request, Response};
use sea_orm::DatabaseConnection;

use volo_gen::person_center::{
    UsersResponse,
    UserResponse,
    Report,
    UserListReq,
    UserDetailReq,
    UpdateUserReq,
    InsertUserReq,
    LoginForm,
    Logged,
    CheckPermissionReq,
    Accessable,
};

use super::service::{
    user_detail_service,
    user_list_service,
    update_user_service,
    delete_user_service,
    insert_user_service,
    login_service,
    check_permission_service,
};

#[derive(Debug, Default)]
pub struct UserService;

impl volo_gen::person_center::User for UserService {
	async fn user_list(&self, req: Request<UserListReq>) -> Result<Response<UsersResponse>, Status> {
        let (_, extensions, data) = req.into_parts();
        let db = extensions.get::<DatabaseConnection>().unwrap();
        Ok(Response::new(user_list_service(data, db).await.unwrap()))
    }

	async fn user_detail(&self, req: Request<UserDetailReq>) -> Result<Response<UserResponse>, Status> {
        let (_, extensions, data) = req.into_parts();
        let db = extensions.get::<DatabaseConnection>().unwrap();
        Ok(Response::new(user_detail_service(data, db).await.unwrap()))
    }

	async fn update_user(&self, req: Request<UpdateUserReq>) -> Result<Response<UserResponse>, Status> {
        let (_, extensions, data) = req.into_parts();
        let db = extensions.get::<DatabaseConnection>().unwrap();
        Ok(Response::new(update_user_service(data, db).await.unwrap()))
    }

	async fn insert_user(&self, req: Request<InsertUserReq>) -> Result<Response<UserResponse>, Status> {
        let (_, extensions, data) = req.into_parts();
        let postgres = extensions.get::<DatabaseConnection>().unwrap();
        let neo4j = extensions.get::<Graph>().unwrap();
        Ok(Response::new(insert_user_service(data, postgres, neo4j).await.unwrap()))
    }

	async fn delete_user(&self, req: Request<UserDetailReq>) -> Result<Response<Report>, Status> {
        let (_, extensions, data) = req.into_parts();
        let postgres = extensions.get::<DatabaseConnection>().unwrap();
        let neo4j = extensions.get::<Graph>().unwrap();
        Ok(Response::new(delete_user_service(data, postgres, neo4j).await.unwrap()))
    }

	async fn login(&self, req: Request<LoginForm>) -> Result<Response<Logged>, Status> {
        let (_, extensions, data) = req.into_parts();
        let db = extensions.get::<DatabaseConnection>().unwrap();
        Ok(Response::new(login_service(data, db).await.unwrap()))
    }

	async fn check_permission(&self, req: Request<CheckPermissionReq>) -> Result<Response<Accessable>, Status> {
        let (_, extensions, data) = req.into_parts();
        let db = extensions.get::<DatabaseConnection>().unwrap();
        Ok(Response::new(check_permission_service(data, db).await.unwrap()))
    }
}
