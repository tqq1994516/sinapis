use std::sync::Arc;

use dapr::{
    appcallback::*,
    dapr::dapr::proto::runtime::v1::app_callback_server::AppCallback,
};
use tonic::{Request, Response, Status};
use sea_orm::DatabaseConnection;

use super::service::{
    user_detail,
    user_list,
    update_user,
    delete_user,
    insert_user,
};

pub struct UserInfo {}

#[tonic::async_trait]
impl AppCallback for UserInfo {
    /// Invokes service method with InvokeRequest.
    async fn on_invoke(
        &self,
        request: Request<InvokeRequest>,
    ) -> Result<Response<InvokeResponse>, Status> {
        let db = &request.extensions().get::<Arc<DatabaseConnection>>().unwrap();
        let r = &request.into_inner();

        let method = &r.method;
        println!("Method: {method}");
        let data = &r.data;
        let data = data.as_ref().unwrap();
        let data = &data.value;
        match method.as_str() {
            "UserDetail" => {
                return Ok(Response::new(user_detail(data, db).await.unwrap()));
            },
            "UserList" => {
                return Ok(Response::new(user_list(data, db).await.unwrap()));
            },
            "UpdateUser" => {
                return Ok(Response::new(update_user(data, db).await.unwrap()));
            },
            "DeleteUser" => {
                return Ok(Response::new(delete_user(data, db).await.unwrap()));
            },
            "InsertUser" => {
                return Ok(Response::new(insert_user(data, db).await.unwrap()));
            },
            _ => return Ok(Response::new(InvokeResponse::default())),
        }
    }

    async fn list_topic_subscriptions(
        &self,
        _request: Request<()>,
    ) -> Result<Response<ListTopicSubscriptionsResponse>, Status> {
        let list_subscriptions = ListTopicSubscriptionsResponse::default();
        Ok(Response::new(list_subscriptions))
    }

    /// Subscribes events from Pubsub.
    async fn on_topic_event(
        &self,
        _request: Request<TopicEventRequest>,
    ) -> Result<Response<TopicEventResponse>, Status> {
        Ok(Response::new(TopicEventResponse::default()))
    }

    /// Lists all input bindings subscribed by this app.
    async fn list_input_bindings(
        &self,
        _request: Request<()>,
    ) -> Result<Response<ListInputBindingsResponse>, Status> {
        Ok(Response::new(ListInputBindingsResponse::default()))
    }

    /// Listens events from the input bindings.
    async fn on_binding_event(
        &self,
        _request: Request<BindingEventRequest>,
    ) -> Result<Response<BindingEventResponse>, Status> {
        Ok(Response::new(BindingEventResponse::default()))
    }
}