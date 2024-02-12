use std::thread;

use actix_web::{error, get, post, web};
use log::warn;
use serde::Deserialize;
use service::{
    archive::{
        resource::{
            create_mod_model, create_provider_model, fetch_downloads, validate_resource_identifier,
            ArchiveProvider,
        },
        task::{
            download_files, parse_language_files, remove_task, save_text_entries,
            update_task_progress, ArchiveTask, ArchiveTaskStage, ARCHIVE_TASKS,
        },
    },
    sea_orm::DatabaseConnection,
};

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateTaskPayload {
    pub provider: ArchiveProvider,
    pub identifier: String,
}

#[post("/tasks")]
/// Creates a new archive task, then executes it in a background thread.
/// If the task is already running, the task ID will be returned.
///
/// ### Payload
/// * provider: ArchiveProvider
/// * identifier: String
///
/// ### Response
/// * String: Task ID
pub async fn create_archive_task(
    state: web::Data<AppState>,
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
        progress: 0.05,
        mc_mod: None,
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
        let result = start_create_task(
            state.db.clone(),
            task_id_clone.clone(),
            payload.provider.clone(),
            payload.identifier.clone(),
            state.config.max_simultaneous_downloads,
        )
        .await;

        if let Err(err) = result {
            let mut tasks = ARCHIVE_TASKS.lock().unwrap();
            let task = tasks.get_mut(&task_id_clone).unwrap();
            task.stage = ArchiveTaskStage::Failed;
            warn!("Execute archive task failed: {:?}", err);
        }
    });

    Ok(task_id)
}

/// Returns the archive task with the given ID.
/// If the task is finished, it will be removed from the task list.
///
/// ### Path parameters
/// * task_id: String
///
/// ### Response
/// * ArchiveTask
///
/// ### Errors
/// * 404: Task not found
#[get("/tasks/{task_id}")]
pub async fn get_archive_task(
    task_id: web::Path<String>,
) -> actix_web::Result<web::Json<ArchiveTask>> {
    let task_id = &task_id.into_inner();
    let tasks = ARCHIVE_TASKS.lock().unwrap();

    if let Some(task) = tasks.get(task_id) {
        if task.stage.is_finished() {
            let task_id = task_id.clone();
            thread::spawn(move || remove_task(&task_id));
        }

        return Ok(web::Json(task.clone()));
    }

    Err(error::ErrorNotFound("Task not found"))
}

async fn start_create_task<'a>(
    db: DatabaseConnection,
    task_id: String,
    provider: ArchiveProvider,
    identifier: String,
    max_simultaneous_downloads: usize,
) -> anyhow::Result<()> {
    // Preparing download list.
    let mut downloads = fetch_downloads(&provider, &identifier).await?;
    downloads.sort_by(|a, b| a.game_version.cmp(&b.game_version));

    // Downloading mod files.
    update_task_progress(&task_id, Some(ArchiveTaskStage::Downloading), 0.1);

    let task_id_clone = task_id.clone();

    download_files(&downloads, max_simultaneous_downloads, |progress| {
        update_task_progress(&task_id_clone, None, 0.1 + progress * 0.75)
    })
    .await?;

    // Extracting and parsing language files.
    update_task_progress(&task_id, Some(ArchiveTaskStage::Extracting), 0.55);
    let text_entries = parse_language_files(&downloads, |progress| {
        update_task_progress(&task_id, None, 0.85 + progress * 0.1)
    })
    .await?;

    // Saving to database.
    update_task_progress(&task_id, Some(ArchiveTaskStage::Saving), 0.95);
    let mc_mod =
        create_mod_model(&db, &provider, identifier.clone(), text_entries.is_empty()).await?;
    create_provider_model(&db, &provider, identifier, mc_mod.id).await?;
    save_text_entries(&db, text_entries, mc_mod.id).await?;

    let mut tasks = ARCHIVE_TASKS.lock().unwrap();
    let task = tasks.get_mut(&task_id).unwrap();
    task.stage = ArchiveTaskStage::Completed;
    task.progress = 1.0;
    task.mc_mod = Some(mc_mod);
    Ok(())
}
