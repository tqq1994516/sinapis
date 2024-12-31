use pingora::{http::RequestHeader, modules::http::{HttpModule, HttpModuleBuilder, Module, ModuleBuilder}, Result, Error, ErrorType::HTTPStatus};
use sea_orm::ConnectOptions;
use std::any::Any;
use http::header::AUTHORIZATION;

#[unsafe(no_mangle)]
pub extern "C" fn export_plugin() -> ModuleBuilder {
    Box::new(IdentificationPluginBuilder {})
}

struct IdentificationPlugin {
    connect_options: ConnectOptions,
}

#[async_trait::async_trait]
impl HttpModule for IdentificationPlugin {
    async fn request_header_filter(&mut self, req: &mut RequestHeader) -> Result<()> {
        println!("uri:{}", req.uri);
        let Some(auth) = req.headers.get(AUTHORIZATION) else {
            return Error::e_explain(HTTPStatus(403), "Auth failed, no auth header");
        };
        Ok(())

        // if auth.as_bytes() != self.credential_header.as_bytes() {
        //     Error::e_explain(HTTPStatus(403), "Auth failed, credential mismatch")
        // } else {
        //     Ok(())
        // }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

struct IdentificationPluginBuilder;

impl HttpModuleBuilder for IdentificationPluginBuilder {
    fn init(&self) -> Module {
        Box::new(IdentificationPlugin {
            connect_options: std::env::var("DATABASE_URL").unwrap().into(),
        })
    }
}
