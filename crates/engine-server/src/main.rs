mod auth;
mod db;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use db::Database;
use engine_core::{ClientMessage, Player, PlayerInput, ServerMessage, WorldState};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::{broadcast, mpsc, Mutex};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

struct AppState {
    next_id: AtomicU32,
    inputs: Mutex<HashMap<u32, PlayerInput>>,
    world: Mutex<WorldState>,
    tx_broadcast: broadcast::Sender<String>,
    db: Mutex<Database>,
}

#[derive(Deserialize)]
struct AuthPayload {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct AuthResponse {
    token: String,
    user: UserDto,
}

#[derive(Serialize)]
struct UserDto {
    id: String,
    username: String,
}

#[derive(Deserialize)]
struct SaveProjectPayload {
    name: String,
    data_json: String,
}

#[derive(Serialize)]
struct ProjectItem {
    id: String,
    name: String,
    created_at: i64,
    updated_at: i64,
}

fn get_now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn extract_user_id(headers: &HeaderMap) -> Result<String, StatusCode> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    auth::verify_jwt(token)
        .map(|claims| claims.sub)
        .map_err(|_| StatusCode::UNAUTHORIZED)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let db_path = "/srv/nyx-engine/data/engine.db";
    let database = Database::init(db_path).expect("Failed to initialize SQLite database");

    let (tx_broadcast, _) = broadcast::channel(100);

    let state = Arc::new(AppState {
        next_id: AtomicU32::new(1),
        inputs: Mutex::new(HashMap::new()),
        world: Mutex::new(WorldState::new(960.0, 540.0)),
        tx_broadcast,
        db: Mutex::new(database),
    });

    // 60 Hz Game Loop
    let loop_state = Arc::clone(&state);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(16));
        let dt = 0.016;

        loop {
            interval.tick().await;

            let mut world = loop_state.world.lock().await;
            let inputs = loop_state.inputs.lock().await;

            world.step(&inputs, dt);

            if world.tick % 2 == 0 && !world.players.is_empty() {
                let sync_msg = ServerMessage::Sync(world.clone());
                if let Ok(json) = serde_json::to_string(&sync_msg) {
                    let _ = loop_state.tx_broadcast.send(json);
                }
            }
        }
    });

    let web_dir = std::path::Path::new("/srv/nyx-engine/web");
    let router = Router::new()
        // API Auth
        .route("/api/auth/register", post(api_register))
        .route("/api/auth/login", post(api_login))
        .route("/api/auth/me", get(api_me))
        // API Projects
        .route("/api/projects", get(api_list_projects).post(api_save_project))
        .route("/api/projects/:id", get(api_get_project).put(api_update_project).delete(api_delete_project))
        // Real-time Engine WebSocket
        .route("/ws", get(ws_handler))
        // Static Web Game Engine Studio
        .nest_service("/", ServeDir::new(web_dir))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 4040));
    println!("🦀 Nyx Studio Game Engine & Backend running at http://0.0.0.0:4040");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

// === AUTH HANDLERS ===

async fn api_register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let username = payload.username.trim().to_string();
    if username.len() < 3 || payload.password.len() < 4 {
        return Err((StatusCode::BAD_REQUEST, "Username (>=3) or Password (>=4) too short".into()));
    }

    let password_hash = auth::hash_password(&payload.password)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let user_id = format!("usr_{}", get_now_ts() * 1000 + (std::process::id() as i64 % 1000));
    let now = get_now_ts();

    let mut db_guard = state.db.lock().await;
    let conn = db_guard.connection();

    let res = conn.execute(
        "INSERT INTO users (id, username, password_hash, created_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![&user_id, &username, &password_hash, now],
    );

    match res {
        Ok(_) => {
            let token = auth::create_jwt(&user_id, &username)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
            Ok(Json(AuthResponse {
                token,
                user: UserDto { id: user_id, username },
            }))
        }
        Err(rusqlite::Error::SqliteFailure(e, _)) if e.code == rusqlite::ffi::ErrorCode::ConstraintViolation => {
            Err((StatusCode::CONFLICT, "Username already taken".into()))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn api_login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let username = payload.username.trim();

    let mut db_guard = state.db.lock().await;
    let conn = db_guard.connection();

    let mut stmt = conn
        .prepare("SELECT id, password_hash FROM users WHERE username = ?1")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut rows = stmt
        .query(rusqlite::params![username])
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(row) = rows.next().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))? {
        let user_id: String = row.get(0).unwrap();
        let hash: String = row.get(1).unwrap();

        if auth::verify_password(&payload.password, &hash) {
            let token = auth::create_jwt(&user_id, username)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
            Ok(Json(AuthResponse {
                token,
                user: UserDto { id: user_id, username: username.to_string() },
            }))
        } else {
            Err((StatusCode::UNAUTHORIZED, "Invalid password".into()))
        }
    } else {
        Err((StatusCode::UNAUTHORIZED, "User not found".into()))
    }
}

async fn api_me(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Result<Json<UserDto>, StatusCode> {
    let user_id = extract_user_id(&headers)?;

    let mut db_guard = state.db.lock().await;
    let conn = db_guard.connection();

    let mut stmt = conn.prepare("SELECT username FROM users WHERE id = ?1").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut rows = stmt.query(rusqlite::params![&user_id]).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(row) = rows.next().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        let username: String = row.get(0).unwrap();
        Ok(Json(UserDto { id: user_id, username }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// === PROJECT MANAGEMENT HANDLERS ===

async fn api_list_projects(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ProjectItem>>, StatusCode> {
    let user_id = extract_user_id(&headers)?;

    let mut db_guard = state.db.lock().await;
    let conn = db_guard.connection();

    let mut stmt = conn
        .prepare("SELECT id, name, created_at, updated_at FROM projects WHERE user_id = ?1 ORDER BY updated_at DESC")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows = stmt
        .query_map(rusqlite::params![&user_id], |row| {
            Ok(ProjectItem {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut list = Vec::new();
    for r in rows.flatten() {
        list.push(r);
    }

    Ok(Json(list))
}

async fn api_save_project(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SaveProjectPayload>,
) -> Result<Json<ProjectItem>, (StatusCode, String)> {
    let user_id = extract_user_id(&headers).map_err(|sc| (sc, "Unauthorized".into()))?;
    let proj_id = format!("prj_{}", get_now_ts() * 1000 + (std::process::id() as i64 % 1000));
    let now = get_now_ts();

    let mut db_guard = state.db.lock().await;
    let conn = db_guard.connection();

    conn.execute(
        "INSERT INTO projects (id, user_id, name, data_json, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![&proj_id, &user_id, &payload.name, &payload.data_json, now, now],
    ).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ProjectItem {
        id: proj_id,
        name: payload.name,
        created_at: now,
        updated_at: now,
    }))
}

async fn api_get_project(
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<String, StatusCode> {
    let user_id = extract_user_id(&headers)?;

    let mut db_guard = state.db.lock().await;
    let conn = db_guard.connection();

    let mut stmt = conn
        .prepare("SELECT data_json FROM projects WHERE id = ?1 AND user_id = ?2")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut rows = stmt
        .query(rusqlite::params![&id, &user_id])
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(row) = rows.next().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        let json_data: String = row.get(0).unwrap();
        Ok(json_data)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn api_update_project(
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SaveProjectPayload>,
) -> Result<StatusCode, StatusCode> {
    let user_id = extract_user_id(&headers)?;
    let now = get_now_ts();

    let mut db_guard = state.db.lock().await;
    let conn = db_guard.connection();

    let rows_affected = conn
        .execute(
            "UPDATE projects SET name = ?1, data_json = ?2, updated_at = ?3 WHERE id = ?4 AND user_id = ?5",
            rusqlite::params![&payload.name, &payload.data_json, now, &id, &user_id],
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if rows_affected > 0 {
        Ok(StatusCode::OK)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn api_delete_project(
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, StatusCode> {
    let user_id = extract_user_id(&headers)?;

    let mut db_guard = state.db.lock().await;
    let conn = db_guard.connection();

    let rows_affected = conn
        .execute("DELETE FROM projects WHERE id = ?1 AND user_id = ?2", rusqlite::params![&id, &user_id])
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if rows_affected > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// === WEBSOCKET GAME ENGINE HANDLER ===

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let player_id = state.next_id.fetch_add(1, Ordering::SeqCst);

    let mut rx_broadcast = state.tx_broadcast.subscribe();

    let (tx_outgoing, mut rx_outgoing) = mpsc::channel::<Message>(32);
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx_outgoing.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let welcome = ServerMessage::Welcome {
        your_id: player_id,
        tick_rate: 60,
    };
    if let Ok(json) = serde_json::to_string(&welcome) {
        let _ = tx_outgoing.send(Message::Text(json)).await;
    }

    let tx_out_clone = tx_outgoing.clone();
    let broadcast_forwarder = tokio::spawn(async move {
        while let Ok(msg) = rx_broadcast.recv().await {
            if tx_out_clone.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let state_clone = Arc::clone(&state);
    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                match client_msg {
                    ClientMessage::Join { name, color } => {
                        let mut world = state_clone.world.lock().await;
                        world.players.insert(player_id, Player::new(player_id, name, color));
                    }
                    ClientMessage::Input(input) => {
                        let mut inputs = state_clone.inputs.lock().await;
                        inputs.insert(player_id, input);
                    }
                }
            }
        }
    }

    broadcast_forwarder.abort();
    send_task.abort();

    {
        let mut world = state.world.lock().await;
        world.players.remove(&player_id);
    }
    {
        let mut inputs = state.inputs.lock().await;
        inputs.remove(&player_id);
    }

    let left_msg = ServerMessage::PlayerLeft { id: player_id };
    if let Ok(json) = serde_json::to_string(&left_msg) {
        let _ = state.tx_broadcast.send(json);
    }
}
