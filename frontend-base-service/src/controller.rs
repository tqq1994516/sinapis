use volo_grpc::{Status, Request, Response};

use volo_gen::frontend_base_service::{
    I18nRequest,
    JsonReply,
};
use volo_gen::google::protobuf::Empty;
use crate::helper::get_config_client;

use super::service::{
    get_route_service,
    get_i18n_service,
};

#[derive(Debug, Default)]
pub struct RouteService {}

#[volo::async_trait]
impl volo_gen::frontend_base_service::Route for RouteService {
	async fn get_route(&self, _: Request<Empty>) -> Result<Response<JsonReply>, Status> {
        let config_client = get_config_client();
        Ok(Response::new(get_route_service(config_client).await.unwrap()))
    }
}

#[derive(Debug, Default)]
pub struct I18nService {}

#[volo::async_trait]
impl volo_gen::frontend_base_service::I18n for I18nService {
	async fn get_i18n(&self, req: Request<I18nRequest>) -> Result<Response<JsonReply>, Status> {
        let req = req.into_inner();
        let config_client = get_config_client();
        Ok(Response::new(get_i18n_service(req, config_client).await.unwrap()))
    }
}
