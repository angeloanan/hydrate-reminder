use crate::{google_fit::create_fit_data_source, storage::InnerAppState};

use {
    crate::{
        http::REQWEST_CLIENT,
        storage::{save_app_state, AppState},
    },
    std::{str::FromStr, sync::LazyLock},
    tauri::api::shell::open,
};

use chrono::Utc;
use reqwest::Url;
use tauri::{AppHandle, Manager, State, WindowBuilder};
use tiny_http::{Response, Server};

const GOOGLE_CLIENT_ID: &str =
    "359154028055-42ip89g4r9m78pgoug1rpropgmbpgfa9.apps.googleusercontent.com";
const GOOGLE_CLIENT_SECRET: &str = "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"; // This is fine since it's a public client

fn generate_oauth_consent_url() -> Url {
    let mut final_url = Url::from_str("https://accounts.google.com/o/oauth2/v2/auth").unwrap();
    let qs = querystring::stringify(vec![
        ("client_id", GOOGLE_CLIENT_ID),
        (
            "scope",
            "https://www.googleapis.com/auth/fitness.nutrition.write",
        ),
        ("response_type", "code"),
        ("redirect_uri", "http://localhost:11132"),
        ("access_type", "offline"),
    ]);

    final_url.set_query(Some(qs.as_str()));
    final_url
}

fn generate_code_exchange_url(code: &str) -> Url {
    let mut final_url = Url::from_str("https://oauth2.googleapis.com/token").unwrap();
    let qs = querystring::stringify(vec![
        ("client_id", GOOGLE_CLIENT_ID),
        ("client_secret", GOOGLE_CLIENT_SECRET),
        ("code", code),
        ("grant_type", "authorization_code"),
        ("redirect_uri", "http://localhost:11132"),
    ]);

    final_url.set_query(Some(qs.as_str()));

    final_url
}

/// Returns a tuple of (`access_token`, `refresh_token`, `expires_in`)
pub async fn exchange_code_with_tokens(code: &str) -> (String, String, i64) {
    let client = REQWEST_CLIENT;
    let url = generate_code_exchange_url(code);

    let fetch_response = client
        .post(url)
        .header("content-length", 0)
        .send()
        .await
        .expect("Unable to exchange OAuth code with token!");
    let response_text = fetch_response
        .text()
        .await
        .expect("Unable to get response text!");
    println!("{response_text}");
    let response_json: serde_json::Value =
        serde_json::from_str(&response_text).expect("Unable to parse JSON response!");

    let access_token = response_json["access_token"].to_string();
    let refresh_token = response_json["refresh_token"].to_string();
    let expires_in = response_json["expires_in"].as_i64().unwrap();

    (access_token, refresh_token, expires_in)
}

/// Returns a tuple of (`access_token`, `expires_in`)
pub async fn refresh_access_token(refresh_token: &str) -> (String, i64) {
    let client = REQWEST_CLIENT;
    let mut final_url = Url::from_str("https://oauth2.googleapis.com/token").unwrap();
    let qs = querystring::stringify(vec![
        ("client_id", GOOGLE_CLIENT_ID),
        ("client_secret", GOOGLE_CLIENT_SECRET),
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
    ]);
    final_url.set_query(Some(&qs));

    let fetch_response = client
        .post(final_url)
        .send()
        .await
        .expect("Unable to refresh access token!");

    let response_json = fetch_response
        .json::<serde_json::Value>()
        .await
        .expect("Unable to parse JSON response!");

    let access_token = response_json["access_token"].to_string();
    let expires_in = response_json["expires_in"].as_i64().unwrap();

    (access_token, expires_in)
}

pub async fn ensure_access_token_validity<'a>(app_state: tauri::State<'a, AppState>) {
    let state = app_state.0.read().unwrap();
    assert!(
        state.google_oauth_access_token.is_some(),
        "Google OAuth access token is missing!"
    );

    let refresh_token = state.google_oauth_refresh_token.as_ref().unwrap();

    // Refresh access token if it's expired
    if Utc::now().timestamp() > state.google_oauth_access_token_expiry_timestamp {
        let (access_token, expires_in) = refresh_access_token(refresh_token).await;

        let mut state = app_state.0.write().unwrap();
        state.google_oauth_access_token = Some(access_token);
        state.google_oauth_access_token_expiry_timestamp = Utc::now().timestamp() + expires_in;
    }
}

// --

#[tauri::command]
pub fn start_oauth_authentication(app: AppHandle) {
    if let Some(window) = app.get_window("oauth") {
        window.set_focus().expect("Unable to focus oauth window!");
    } else {
        WindowBuilder::new(&app, "oauth", tauri::WindowUrl::App("oauth".into()))
            .resizable(false)
            .closable(true)
            .build()
            .expect("Unable to create a new window!");

        let oauth_url = generate_oauth_consent_url();
        open(&app.shell_scope(), oauth_url.as_str(), None).unwrap();

        // Spawn OAuth server
        let app_handle = app.app_handle();
        tokio::task::spawn(launch_redirect_server(app_handle));
    }
}

pub async fn launch_redirect_server(app: AppHandle) {
    let http_server = Server::http("localhost:11132").unwrap();
    println!("HTTP Server now listening on {}", http_server.server_addr());

    for request in http_server.incoming_requests() {
        let req_url = request.url();
        println!("Received request: {req_url}");

        if !req_url.starts_with("/?") {
            request
                .respond(Response::from_string("404 Not Found").with_status_code(404))
                .ok();

            continue;
        }

        let qs = querystring::querify(&req_url[2..]);
        let qs_code_tuple = qs.iter().find(|(k, _v)| *k == "code");

        if qs_code_tuple.is_none() {
            request
                .respond(Response::from_string("400 Bad Request").with_status_code(400))
                .ok();

            continue;
        }

        let code = qs_code_tuple.unwrap().1.to_string();
        println!("Received code: {code:?}");
        app.get_window("oauth").and_then(|w| w.close().ok());

        request
            .respond(tiny_http::Response::from_string(
                "Completed! You may close this window",
            ))
            .ok();

        // Spawn a task to exchange code with tokens
        let app_state = app.state::<AppState>();
        let (access_token, refresh_token, expires_in) = exchange_code_with_tokens(&code).await;
        let expire_timestamp = chrono::Utc::now().timestamp() + expires_in;

        let fit_data_source_id = create_fit_data_source(&access_token).await;

        {
            let mut state = app_state.0.write().unwrap();
            state.google_oauth_refresh_token = Some(refresh_token);
            state.google_oauth_access_token = Some(access_token);
            state.google_oauth_access_token_expiry_timestamp = expire_timestamp;
            state.google_fit_data_source_id = Some(fit_data_source_id);
        }

        save_app_state(&app_state.0.read().unwrap()).unwrap();

        break;
    }

    println!("Shutting down HTTP Server");
    http_server.unblock();
    std::mem::drop(http_server);
}
