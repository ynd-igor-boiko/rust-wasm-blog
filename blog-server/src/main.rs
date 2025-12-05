mod application;
mod data;
mod domain;
mod infrastructure;
mod presentation;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use std::sync::Arc;
use tonic::transport::Server;
use tracing::{error, info};

use application::{AuthService, BlogService};
use data::{PostgresPostRepository, PostgresUserRepository};
use infrastructure::{create_pool, init_logging, run_migrations, JwtService};
use presentation::grpc_service::blog_proto::blog_service_server::BlogServiceServer;
use presentation::{configure_routes, BlogGrpcService};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_logging();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL required");
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET required");

    info!("connecting to postgres");
    let pool = create_pool(&db_url).await?;
    run_migrations(&pool).await?;
    info!("migrations applied");

    let jwt = JwtService::new(&jwt_secret);
    let user_repo = PostgresUserRepository::new(pool.clone());
    let post_repo = PostgresPostRepository::new(pool);

    let auth_svc = Arc::new(AuthService::new(user_repo, jwt));
    let blog_svc = Arc::new(BlogService::new(post_repo));

    let auth_for_grpc = auth_svc.clone();
    let blog_for_grpc = blog_svc.clone();

    let http = HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600),
            )
            .app_data(web::Data::new(auth_svc.clone()))
            .app_data(web::Data::new(blog_svc.clone()))
            .configure(configure_routes)
    })
    .bind("0.0.0.0:3000")?
    .run();

    let grpc = Server::builder()
        .add_service(BlogServiceServer::new(BlogGrpcService::new(auth_for_grpc, blog_for_grpc)))
        .serve("0.0.0.0:50051".parse()?);

    info!("http server on :3000, grpc on :50051");

    tokio::select! {
        res = http => {
            if let Err(e) = res {
                error!("http server crashed: {}", e);
            }
        }
        res = grpc => {
            if let Err(e) = res {
                error!("grpc server crashed: {}", e);
            }
        }
    }

    Ok(())
}
