use pool::dapr::person_center::GrpcClientManager;

pub struct Middleware {
    dapr_grpc_client: GrpcClientManager,
}