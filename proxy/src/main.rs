use async_trait::async_trait;
use http::header::HOST;
use http::{HeaderValue, Uri};
use pingora::modules::http::{HttpModules, ModuleBuilder};
use pingora::prelude::*;
use std::sync::Arc;
use libloading::{Library, Symbol};
use std::env;

fn load_plugin(lib_name: &str) -> Result<Library, libloading::Error> {
    if let Ok(current_dir) = env::current_dir() {
        let lib = unsafe { Library::new(current_dir.parent().unwrap().join("target").join("release").join(lib_name)) }?;
        // 可以进一步获取插件中的符号（函数、变量等）
        Ok(lib)
    } else {
        Err(libloading::Error::DlOpenUnknown)
    }
}


fn main() {
    let mut my_server = Server::new(Some(Opt::parse_args())).unwrap();
    my_server.bootstrap();

    let project_dir = std::env::current_dir().unwrap();
    // 加载环境变量
    dotenv::from_path(project_dir.parent().unwrap().join("entity").join(".env")).unwrap();

    let mut upstreams =
        LoadBalancer::try_from_iter(["1.1.1.1:443", "1.0.0.1:443"]).unwrap();

    let hc = TcpHealthCheck::new();
    upstreams.set_health_check(hc);
    upstreams.health_check_frequency = Some(std::time::Duration::from_secs(1));

    let background = background_service("health check", upstreams);
    let upstreams = background.task();
    // for backend in upstreams.backends().get_backend().iter() {
    //     if upstreams.backends().ready(backend) {
    //         let naming_service = NamingServiceBuilder::new(
    //             ClientProps::new()
    //                 .server_addr("10.19.19.26:18848")
    //                 // Attention! "public" is "", it is recommended to customize the namespace with clear meaning.
    //                 .namespace("pingora")
    //                 .app_name("simple_app")
    //                 .auth_username("nacos")
    //                 .auth_password("Tqq1994516!")
    //         )
    //         .enable_auth_plugin_http()
    //         .build()?;
    //     }
    // }

    // `upstreams` no longer need to be wrapped in an arc
    let mut lb = http_proxy_service(&my_server.configuration, LB(upstreams));
    lb.add_tcp("0.0.0.0:6188");

    my_server.add_service(background);

    my_server.add_service(lb);
    my_server.run_forever();
}

pub struct LB(Arc<LoadBalancer<RoundRobin>>);

pub struct ProxyCtx {
    pub uri: Uri,
    pub host: HeaderValue,
}

#[async_trait]
impl ProxyHttp for LB {
    /// For this small example, we don't need context storage
    type CTX = ProxyCtx;
    fn new_ctx(&self) -> Self::CTX {
        ProxyCtx { uri: Uri::default(), host: HeaderValue::from_static("") }
    }

    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool> {
        ctx.uri = session.req_header().uri.clone();
        ctx.host = session.req_header().headers.get(HOST).unwrap().clone();
        Ok(false)
    }

    async fn upstream_peer(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<Box<HttpPeer>> {
        let upstream = self.0
            .select(b"", 256) // hash doesn't matter for round robin
            .unwrap();

        println!("upstream peer is: {upstream:?}");

        // Set SNI to one.one.one.one
        let peer = Box::new(HttpPeer::new(upstream, true, "one.one.one.one".to_string()));
        Ok(peer)
    }

    async fn upstream_request_filter(
        &self,
        session: &mut Session,
        upstream_request: &mut RequestHeader,
        ctx: &mut (),
    ) -> Result<()> {
        println!("upstream request filter");
        Ok(())
    }
}
