use axum::{
    extract::{State, Form},
    response::{Html, Redirect},
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc, Duration as ChronoDuration};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Duration,
    fs,
    path::Path,
};
use tokio::time;
use tower_http::services::ServeDir;
use askama::Template;

// Добавляем модуль с фильтром для форматирования чисел с плавающей точкой
mod filters {
    pub fn format_float(f: &f64) -> ::askama::Result<String> {
        Ok(format!("{:.4}", f))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Status {
    Up,
    Down,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Resource {
    url: String,
    status: Status,
    status_code: Option<u16>,
    last_checked: DateTime<Utc>,
    response_time: Option<u64>,
    response_times: Vec<u64>,
    jitter: Option<f64>,
    #[serde(skip)]
    minutes_ago: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppConfig {
    check_interval: u64,  // In seconds
    refresh_interval: u64, // In seconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SavedState {
    resources: HashMap<String, Resource>,
    config: AppConfig,
}

const CONFIG_PATH: &str = "webcheck_config.json";

type AppState = Arc<RwLock<(HashMap<String, Resource>, AppConfig)>>;

// Функция-помощник для вычисления времени "n минут назад"
fn time_ago_in_minutes(date: &DateTime<Utc>) -> i64 {
    let now = Utc::now();
    let diff = now.signed_duration_since(*date);
    diff.num_minutes()
}

#[derive(Template)]
#[template(path = "index.html")]
#[template(filters(filters))]  // Указываем модуль с фильтрами
struct IndexTemplate {
    resources: Vec<Resource>,
    config: AppConfig,
}

async fn check_resource(url: String, state: AppState) {
    let client = reqwest::Client::new();
    
    let start = std::time::Instant::now();
    let result = client.get(&url).timeout(Duration::from_secs(5)).send().await;
    let elapsed = start.elapsed().as_millis() as u64;
    
    let (status, status_code) = match result {
        Ok(response) => {
            let status_code = response.status().as_u16();
            if response.status().is_success() {
                (Status::Up, Some(status_code))
            } else {
                (Status::Down, Some(status_code))
            }
        },
        Err(_) => (Status::Down, None),
    };
    
    // Update or create resource
    let mut state_guard = state.write().unwrap();
    let (resources, _) = &mut *state_guard;
    
    let resource = if let Some(mut existing) = resources.get(&url).cloned() {
        // Update existing resource
        existing.status = status;
        existing.status_code = status_code;
        existing.last_checked = Utc::now();
        existing.response_time = Some(elapsed);
        
        // Update response times history (keep last 10)
        existing.response_times.push(elapsed);
        if existing.response_times.len() > 10 {
            existing.response_times.remove(0);
        }
        
        // Calculate jitter (standard deviation of response times)
        if existing.response_times.len() >= 2 {
            let mean: f64 = existing.response_times.iter().sum::<u64>() as f64 / existing.response_times.len() as f64;
            let variance: f64 = existing.response_times.iter()
                .map(|&x| {
                    let diff = x as f64 - mean;
                    diff * diff
                })
                .sum::<f64>() / existing.response_times.len() as f64;
            existing.jitter = Some(variance.sqrt());
        }
        
        existing
    } else {
        // Create new resource
        Resource {
            url: url.clone(),
            status,
            status_code,
            last_checked: Utc::now(),
            response_time: Some(elapsed),
            response_times: vec![elapsed],
            jitter: None,
            minutes_ago: 0,
        }
    };
    
    resources.insert(url, resource);
    
    // Save state periodically (to avoid too many writes, only save every 10 minutes)
    let now = Utc::now();
    let save_interval = ChronoDuration::minutes(10);
    
    static mut LAST_SAVE: Option<DateTime<Utc>> = None;
    
    let should_save = unsafe {
        match LAST_SAVE {
            Some(last_save) => now.signed_duration_since(last_save) >= save_interval,
            None => true,
        }
    };
    
    if should_save {
        // Drop the guard before saving to avoid deadlocks
        drop(state_guard);
        save_state(&state);
        
        unsafe {
            LAST_SAVE = Some(now);
        }
    }
}

async fn index_handler(State(state): State<AppState>) -> Html<String> {
    let state_guard = state.read().unwrap();
    let (resources, config) = &*state_guard;
    
    // Клонируем ресурсы и рассчитываем время в минутах
    let mut resources_vec: Vec<Resource> = resources.values().cloned().collect();
    
    for resource in &mut resources_vec {
        resource.minutes_ago = time_ago_in_minutes(&resource.last_checked);
    }
    
    let template = IndexTemplate {
        resources: resources_vec,
        config: config.clone(),
    };
    
    Html(template.render().unwrap_or_else(|_| "Error rendering template".to_string()))
}

#[derive(Debug, Clone, Deserialize)]
struct AddResourceForm {
    url: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ConfigForm {
    check_interval: u64,
    refresh_interval: u64,
}

async fn add_resource(
    State(state): State<AppState>,
    Form(form): Form<AddResourceForm>,
) -> Redirect {
    // Validate URL
    if !form.url.starts_with("http://") && !form.url.starts_with("https://") {
        return Redirect::to("/");
    }
    
    // Add resource to state with unknown status
    let mut state_guard = state.write().unwrap();
    let (resources, _) = &mut *state_guard;
    
    if !resources.contains_key(&form.url) {
        let resource = Resource {
            url: form.url.clone(),
            status: Status::Unknown,
            status_code: None,
            last_checked: Utc::now(),
            response_time: None,
            response_times: Vec::new(),
            jitter: None,
            minutes_ago: 0,
        };
        
        resources.insert(form.url.clone(), resource);
        
        // Save state after adding resource
        drop(state_guard);
        save_state(&state);
        
        // Spawn a task to check this resource
        let state_clone = state.clone();
        let url_clone = form.url.clone();
        tokio::spawn(async move {
            check_resource(url_clone, state_clone).await;
        });
    } else {
        drop(state_guard);
    }
    
    Redirect::to("/")
}

async fn remove_resource(
    State(state): State<AppState>,
    Form(form): Form<AddResourceForm>,
) -> Redirect {
    let mut state_guard = state.write().unwrap();
    let (resources, _) = &mut *state_guard;
    resources.remove(&form.url);
    
    // Save state after removing resource
    drop(state_guard);
    save_state(&state);
    
    Redirect::to("/")
}

async fn update_config(
    State(state): State<AppState>,
    Form(form): Form<ConfigForm>,
) -> Redirect {
    // Update config, ensure reasonable limits
    let check_interval = form.check_interval.max(5).min(3600);
    let refresh_interval = form.refresh_interval.max(5).min(3600);
    
    // Ограничиваем область видимости блокировки
    {
        let mut state_guard = state.write().unwrap();
        let (_, config) = &mut *state_guard;
        
        config.check_interval = check_interval;
        config.refresh_interval = refresh_interval;
    } // write-блокировка освобождается здесь
    
    // Теперь save_state может получить read-блокировку без проблем
    save_state(&state);
    
    Redirect::to("/")
}

// Save current state to file
fn save_state(state: &AppState) {
    let state_guard = state.read().unwrap();
    let (resources, config) = &*state_guard;
    
    let saved_state = SavedState {
        resources: resources.clone(),
        config: config.clone(),
    };
    
    match serde_json::to_string_pretty(&saved_state) {
        Ok(json) => {
            if let Err(e) = fs::write(CONFIG_PATH, json) {
                eprintln!("Failed to save state: {}", e);
            }
        },
        Err(e) => eprintln!("Failed to serialize state: {}", e),
    }
}

// Load state from file
fn load_state() -> Option<SavedState> {
    if !Path::new(CONFIG_PATH).exists() {
        return None;
    }
    
    match fs::read_to_string(CONFIG_PATH) {
        Ok(json) => {
            match serde_json::from_str(&json) {
                Ok(state) => Some(state),
                Err(e) => {
                    eprintln!("Failed to parse saved state: {}", e);
                    None
                }
            }
        },
        Err(e) => {
            eprintln!("Failed to read saved state: {}", e);
            None
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Define default resources to monitor (used only if no saved state)
    let default_urls = vec![
        "https://www.google.com".to_string(),
        "https://www.github.com".to_string(),
        "https://www.rust-lang.org".to_string(),
        "https://www.wikipedia.org".to_string(),
        "https://www.microsoft.com".to_string(),
    ];
    
    // Initialize default config
    let default_config = AppConfig {
        check_interval: 60,   // Default: check every 60 seconds
        refresh_interval: 30, // Default: refresh UI every 30 seconds
    };
    
    // Try to load saved state or use defaults
    let (resources, config) = match load_state() {
        Some(saved_state) => {
            println!("Loaded saved configuration");
            (saved_state.resources, saved_state.config)
        },
        None => {
            println!("Using default configuration");
            let mut resources = HashMap::new();
            
            // Initialize resources with Unknown status
            for url in &default_urls {
                let resource = Resource {
                    url: url.clone(),
                    status: Status::Unknown,
                    status_code: None,
                    last_checked: Utc::now(),
                    response_time: None,
                    response_times: Vec::new(),
                    jitter: None,
                    minutes_ago: 0,
                };
                resources.insert(url.clone(), resource);
            }
            
            (resources, default_config)
        }
    };
    
    // Create state
    let state: AppState = Arc::new(RwLock::new((resources, config)));
    
    // Clone state for the background task
    let background_state = state.clone();
    
    // Spawn background task to check resources
    tokio::spawn(async move {
        loop {
            // Get current check interval
            let check_interval = {
                let state_guard = background_state.read().unwrap();
                let (_, config) = &*state_guard;
                config.check_interval
            };
            
            let mut interval = time::interval(Duration::from_secs(check_interval));
            interval.tick().await;  // First tick completes immediately
            
            // Make a copy of URLs to avoid holding the lock while checking
            let urls_to_check = {
                let state_guard = background_state.read().unwrap();
                let (resources, _) = &*state_guard;
                resources.keys().cloned().collect::<Vec<String>>()
            };
            
            for url in urls_to_check {
                let state_clone = background_state.clone();
                tokio::spawn(async move {
                    check_resource(url, state_clone).await;
                });
            }
            
            // Wait for next interval
            interval.tick().await;
        }
    });
    
    // Do initial check of all resources
    {
        let state_guard = state.read().unwrap();
        let (resources, _) = &*state_guard;
        let initial_urls: Vec<String> = resources.keys().cloned().collect();
        
        for url in initial_urls {
            let state_clone = state.clone();
            tokio::spawn(async move {
                check_resource(url, state_clone).await;
            });
        }
    }
    
    // Build router
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/add", post(add_resource))
        .route("/remove", post(remove_resource))
        .route("/config", post(update_config))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);
    
    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running at http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
