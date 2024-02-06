/*
 * Author: Ruben Fiszel
 * Copyright: Windmill Labs, Inc 2022
 * This file and its contents are licensed under the AGPLv3 License.
 * Please see the included NOTICE for copyright information and
 * LICENSE-AGPL for a copy of the license.
 */

use axum::{
    extract::{Extension, Path},
    routing::{get, post},
    Json, Router,
};

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use windmill_audit::{audit_log, ActionKind};
use windmill_common::{
    error::{self},
    DB,
};

use crate::{db::ApiAuthed, utils::require_super_admin};

pub fn global_service() -> Router {
    Router::new()
        .route("/list_worker_groups", get(list_worker_groups))
        .route("/update/:name", post(update_config).delete(delete_config))
        .route("/get/:name", get(get_config))
}

#[derive(Serialize, Deserialize, FromRow)]
struct Config {
    name: String,
    config: serde_json::Value,
}

async fn list_worker_groups(Extension(db): Extension<DB>) -> error::JsonResult<Vec<Config>> {
    let rows = sqlx::query_as!(Config, "SELECT * FROM config WHERE name LIKE 'worker__%'")
        .fetch_all(&db)
        .await?;
    Ok(Json(rows))
}

async fn get_config(
    authed: ApiAuthed,
    Path(name): Path<String>,
    Extension(db): Extension<DB>,
) -> error::JsonResult<Option<serde_json::Value>> {
    require_super_admin(&db, &authed.email).await?;

    let config = sqlx::query_as!(Config, "SELECT * FROM config WHERE name = $1", name)
        .fetch_optional(&db)
        .await?
        .map(|c| c.config);

    Ok(Json(config))
}

async fn update_config(
    Path(name): Path<String>,
    Extension(db): Extension<DB>,
    authed: ApiAuthed,
    Json(config): Json<serde_json::Value>,
) -> error::Result<String> {
    require_super_admin(&db, &authed.email).await?;

    #[cfg(not(feature = "enterprise"))]
    if name.starts_with("worker__") {
        return Err(error::Error::BadRequest(
            "Worker groups configurable from UI available only in the enterprise version"
                .to_string(),
        ));
    }

    let mut tx = db.begin().await?;
    sqlx::query!(
        "INSERT INTO config (name, config) VALUES ($1, $2) ON CONFLICT (name) DO UPDATE SET config = $2",
        &name,
        config
    )
    .execute(&mut *tx)
    .await?;

    audit_log(
        &mut *tx,
        &authed.username,
        "worker_config.update",
        ActionKind::Update,
        "global",
        Some(&name),
        None,
    )
    .await?;
    tx.commit().await?;
    Ok(format!("Updated config {name}"))
}

async fn delete_config(
    Path(name): Path<String>,
    Extension(db): Extension<DB>,
    authed: ApiAuthed,
) -> error::Result<String> {
    require_super_admin(&db, &authed.email).await?;

    let mut tx = db.begin().await?;

    let deleted = sqlx::query!("DELETE FROM config WHERE name = $1 RETURNING name", name)
        .fetch_all(&db)
        .await?;

    audit_log(
        &mut *tx,
        &authed.username,
        "worker_config.delete",
        ActionKind::Delete,
        "global",
        Some(&name),
        None,
    )
    .await?;
    tx.commit().await?;
    if deleted.len() == 0 {
        return Err(error::Error::NotFound(format!(
            "Config {name} not found",
            name = name
        )));
    }
    Ok(format!("Deleted config {name}"))
}
