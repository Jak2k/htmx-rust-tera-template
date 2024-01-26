use actix_web::{get, web, HttpServer, Responder};
use color_eyre::eyre::Result;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Mutex};
use tera::Tera;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        match Tera::new("src/templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Template parsing error(s): {e}");
                ::std::process::exit(1);
            }
        }
    };
}

#[derive(Serialize, Deserialize)]
struct AppState {
    counter: Mutex<HashMap<String, i32>>,
}

impl AppState {
    fn increment(&self, id: &str) {
        let mut counter = self.counter.lock().unwrap();

        if counter.contains_key(id) {
            let count = counter.get_mut(id).unwrap();
            *count += 1;
        } else {
            counter.insert(id.to_string(), 0);
        }
    }

    fn decrement(&self, id: &str) {
        let mut counter = self.counter.lock().unwrap();

        if counter.contains_key(id) {
            let count = counter.get_mut(id).unwrap();
            *count -= 1;
        } else {
            counter.insert(id.to_string(), 0);
        }
    }
}

#[get("/")]
async fn index(state: actix_web::web::Data<AppState>) -> impl Responder {
    let mut ctx = tera::Context::new();
    ctx.insert("counter", &*state.counter.lock().unwrap());

    let rendered = TEMPLATES.render("index.html", &ctx).unwrap();

    actix_web::HttpResponse::Ok().body(rendered)
}

#[get("/counter/{id}/{action}")]
async fn counter_handler(
    action: actix_web::web::Path<(String, String)>,
    state: actix_web::web::Data<AppState>,
) -> impl Responder {
    let (id, action) = action.into_inner();

    let mut ctx = tera::Context::new();

    match action.as_str() {
        "increment" => {
            state.increment(&id);
        }
        "decrement" => {
            state.decrement(&id);
        }
        _ => {
            return actix_web::HttpResponse::BadRequest().body("Invalid action");
        }
    }

    let counter = state.counter.lock().unwrap();
    let count = counter.get(&id).unwrap_or(&0);
    ctx.insert("count", count);
    ctx.insert("id", &id);

    let rendered = TEMPLATES.render("counter.html", &ctx).unwrap();

    actix_web::HttpResponse::Ok().body(rendered)
}

lazy_static! {
    static ref DEFAULT_COUNTERS: Vec<(String, i32)> =
        vec![("c1".to_string(), 0), ("c2".to_string(), 10)];
}

#[actix_web::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    HttpServer::new(|| {
        actix_web::App::new()
            // register state
            .app_data(web::Data::new(AppState {
                counter: Mutex::new(DEFAULT_COUNTERS.clone().into_iter().collect()),
            }))
            // index route
            .service(index)
            // counter route
            .service(counter_handler)
            // static files
            .service(actix_files::Files::new("/", "./src/static/").show_files_listing())
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await?;

    Ok(())
}
