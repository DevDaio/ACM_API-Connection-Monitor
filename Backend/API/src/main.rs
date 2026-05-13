use axum::{routing::{get, post, put, delete},Router,};
use postgres::{Client, NoTls};

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/acm", get(healtcheck))
        .route("/acm/createAccount", post(healtcheck))
        .route("/acm/home", get(healtcheck))
        .route("/acm/user", get(healtcheck))
        .route("/acm/user/changePassword", put(athing))
        .route("/acm/user/changeEmail", put(athing))
        .route("/acm/user/deleteAccount", delete(healtcheck))
        .route("/acm/addEndpoint", put(athing))
        .route("/acm/setIntervall", put(athing))
        .route("/acm/deleteConfirm", put(athing))
        .route("/acm/log", get(healtcheck));
        
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn healtcheck() -> String {
    String::from("Hello World")
}

async fn athing() -> String {
    
    String::from(" A thing Erfolgreich angelget")
}