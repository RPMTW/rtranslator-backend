use std::{collections::HashMap, sync::Mutex};

use actix_web::{error, get, post, web};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use service::archive::resource::{
    fetch_download_urls, validate_resource_identifier, ArchiveProvider,
};

lazy_static! {
    pub static ref ARCHIVE_TASKS: Mutex<HashMap<String, ArchiveTask>> = Mutex::new(HashMap::new());
}

#[derive(Debug, Serialize, Clone)]
pub struct ArchiveTask {
    pub stage: ArchiveTaskStage,
    pub progress: f32,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ArchiveTaskStage {
    Preparing,
    Downloading,
    Extracting,
    Parsing,
    Saving,
    Completed,
    Failed,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskPayload {
    pub provider: ArchiveProvider,
    pub identifier: String,
}

#[post("/tasks")]
/**
 * Creates a new archive task.
 *
 * Payload:
 *   provider: ArchiveProvider
 *   identifier: String
 *
 * Response:
 *   String: Task ID
 */
pub async fn create_archive_task(
    payload: web::Json<CreateTaskPayload>,
) -> actix_web::Result<String> {
    let identifier_valid = validate_resource_identifier(&payload.provider, &payload.identifier)
        .await
        .map_or(false, |f| f);
    if !identifier_valid {
        return Err(error::ErrorBadRequest("Invalid resource identifier"));
    }

    let task_id = format!(
        "{}-{}",
        serde_json::to_value(&payload.provider)
            .unwrap()
            .as_str()
            .unwrap(),
        &payload.identifier
    );
    let task = ArchiveTask {
        stage: ArchiveTaskStage::Preparing,
        progress: 0.0,
    };

    {
        let mut tasks = ARCHIVE_TASKS.lock().unwrap();
        // The task is already running.
        if tasks.contains_key(&task_id) {
            return Ok(task_id);
        }
        tasks.insert(task_id.clone(), task);
    }

    let task_id_clone = task_id.clone();
    tokio::spawn(async move {
        start_create_task(
            task_id_clone,
            payload.provider.clone(),
            payload.identifier.clone(),
        )
        .await;
    });

    Ok(task_id)
}

#[get("/tasks/{task_id}")]
pub async fn get_archive_task(
    task_id: web::Path<String>,
) -> actix_web::Result<web::Json<ArchiveTask>> {
    let tasks = ARCHIVE_TASKS.lock().unwrap();
    println!("Tasks: {:?}", tasks);
    let task = tasks.get(&task_id.into_inner());

    match task {
        Some(task) => Ok(web::Json(task.clone())),
        None => Err(error::ErrorNotFound("Task not found")),
    }
}

async fn start_create_task(task_id: String, provider: ArchiveProvider, identifier: String) {
    let download_urls = fetch_download_urls(&provider, &identifier).await;
    {
        let mut tasks = ARCHIVE_TASKS.lock().unwrap();
        let task = tasks.get_mut(&task_id).unwrap();
        task.stage = ArchiveTaskStage::Downloading;
        task.progress = 0.1;
    }

    // unimplemented!()
}
