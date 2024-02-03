use std::sync::LazyLock;

use reqwest::Url;
use tauri::{AppHandle, Manager, WindowBuilder};
use tiny_http::{Response, Server};

const GOOGLE_OAUTH_URL: LazyLock<Url> = LazyLock::new(|| {
    let qs = querystring::stringify(vec![
        (
            "client_id",
            "359154028055-42ip89g4r9m78pgoug1rpropgmbpgfa9.apps.googleusercontent.com",
        ),
        (
            "scope",
            "https://www.googleapis.com/auth/fitness.nutrition.write",
        ),
        ("prompt", "consent"),
        ("response_type", "code"),
        ("redirect_uri", "http://localhost:11132"),
        ("access_type", "online"),
    ]);

    Url::parse(format!("https://accounts.google.com/o/oauth2/v2/auth?{}", qs).as_str()).unwrap()
});

#[tauri::command]
pub fn start_oauth_authentication(app: AppHandle) {
    if let Some(window) = app.get_window("oauth") {
        window.set_focus().expect("Unable to focus oauth window!")
    } else {
        WindowBuilder::new(&app, "oauth", tauri::WindowUrl::App("oauth".into()))
            .hidden_title(true)
            .resizable(false)
            .closable(true)
            .build()
            .expect("Unable to create a new window!");

        open::that_detached(GOOGLE_OAUTH_URL.as_str()).unwrap();

        // Spawn OAuth server
        tokio::task::spawn(redirect_server(app.app_handle()));
    }
}

pub async fn redirect_server(app: AppHandle) {
    let http_server = Server::http("localhost:11132").unwrap();
    println!("HTTP Server now listening on {}", http_server.server_addr());

    for request in http_server.incoming_requests() {
        let req_url = request.url();
        println!("Received request: {}", req_url);

        if !req_url.starts_with("/?") {
            request
                .respond(Response::from_string("404 Not Found").with_status_code(404))
                .ok();

            continue;
        }

        let qs = querystring::querify(&req_url[2..]);
        let code = qs.iter().find(|(k, _v)| *k == "code");

        if code.is_none() {
            request
                .respond(Response::from_string("400 Bad Request").with_status_code(400))
                .ok();

            continue;
        }

        println!("Received code: {}", code.unwrap().1);
        app.get_window("oauth").and_then(|w| w.close().ok());

        request
            .respond(tiny_http::Response::from_string(
                "Loading... You may close this window",
            ))
            .ok();

        break;
    }

    println!("Shutting down HTTP Server");
    http_server.unblock();
    std::mem::drop(http_server);
}
