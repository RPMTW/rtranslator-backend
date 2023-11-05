use actix_web::{error, get, post, web};
use serde::Deserialize;
use service::archive::{
    resource::{fetch_downloads, validate_resource_identifier, ArchiveProvider},
    task::{download_files, extract_files, ArchiveTask, ArchiveTaskStage, ARCHIVE_TASKS},
};

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
        let result = start_create_task(
            task_id_clone.clone(),
            payload.provider.clone(),
            payload.identifier.clone(),
        )
        .await;

        if let Err(err) = result {
            let mut tasks = ARCHIVE_TASKS.lock().unwrap();
            let task = tasks.get_mut(&task_id_clone).unwrap();
            task.stage = ArchiveTaskStage::Failed;
            println!("Execute archive task failed: {:?}", err);
        }
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

async fn start_create_task(
    task_id: String,
    provider: ArchiveProvider,
    identifier: String,
) -> anyhow::Result<()> {
    let downloads = fetch_downloads(&provider, &identifier).await?;
    {
        let mut tasks = ARCHIVE_TASKS.lock().unwrap();
        let task = tasks.get_mut(&task_id).unwrap();
        task.stage = ArchiveTaskStage::Downloading;
        task.progress = 0.05;
    }

    let file_locations = download_files(&task_id, downloads).await?;
    {
        let mut tasks = ARCHIVE_TASKS.lock().unwrap();
        let task = tasks.get_mut(&task_id).unwrap();
        task.stage = ArchiveTaskStage::Extracting;
        task.progress = 0.55;
    }

    let result = extract_files(file_locations).await?;
    {
        let mut tasks = ARCHIVE_TASKS.lock().unwrap();
        let task = tasks.get_mut(&task_id).unwrap();
        task.stage = ArchiveTaskStage::Saving;
        task.progress = 0.9;
    }

    // save
    {
        let mut tasks = ARCHIVE_TASKS.lock().unwrap();
        let task = tasks.get_mut(&task_id).unwrap();
        task.stage = ArchiveTaskStage::Completed;
        task.progress = 1.0;
    }

    Ok(())
}
