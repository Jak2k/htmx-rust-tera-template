use actix_web::{get, HttpServer, Responder};
use color_eyre::eyre::Result;
use lazy_static::lazy_static;
use tera::Tera;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let tera = match Tera::new("src/templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Template parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera
    };
}

#[get("/")]
async fn index() -> impl Responder {
    let mut ctx = tera::Context::new();
    ctx.insert("name", "Tera");

    let rendered = TEMPLATES.render("index.html", &ctx).unwrap();

    actix_web::HttpResponse::Ok().body(rendered)
}

#[actix_web::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    HttpServer::new(|| {
        actix_web::App::new()
            .service(index)
            // serve static files
            .service(actix_files::Files::new("/", "./src/static/").show_files_listing())
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await?;

    Ok(())
}
