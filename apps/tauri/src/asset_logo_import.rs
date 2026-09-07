//! One-time import of custom asset logos from the pre-3.8 file-based logo
//! store into the SQLite-backed `AssetLogoService`.
//!
//! Drop PNG files named `<asset_id>.png` into `<app_data>/asset-logos-import/`.
//! On startup every file is upserted through the service (so validation and
//! sync bookkeeping apply), then the folder is renamed to
//! `asset-logos-import.done` so the import never runs twice.

use std::path::Path;
use std::sync::Arc;

use base64::Engine;
use wealthfolio_core::assets::{AssetLogoServiceTrait, UpsertAssetLogo};

const IMPORT_DIR: &str = "asset-logos-import";
const DONE_DIR: &str = "asset-logos-import.done";

pub async fn import_pending_logos(app_data_dir: &str, service: Arc<dyn AssetLogoServiceTrait>) {
    let dir = Path::new(app_data_dir).join(IMPORT_DIR);
    if !dir.is_dir() {
        return;
    }

    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(e) => {
            log::warn!("asset logo import: cannot read {}: {e}", dir.display());
            return;
        }
    };

    let (mut imported, mut failed) = (0usize, 0usize);
    for path in entries.flatten().map(|entry| entry.path()) {
        if path.extension().and_then(|ext| ext.to_str()) != Some("png") {
            continue;
        }
        let Some(asset_id) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };

        let outcome = match std::fs::read(&path) {
            Ok(bytes) => {
                let data_base64 = base64::engine::general_purpose::STANDARD.encode(bytes);
                service
                    .upsert_asset_logo(asset_id, UpsertAssetLogo { data_base64 })
                    .await
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            }
            Err(e) => Err(e.to_string()),
        };

        match outcome {
            Ok(()) => imported += 1,
            Err(e) => {
                failed += 1;
                log::warn!("asset logo import: {asset_id}: {e}");
            }
        }
    }

    log::info!("asset logo import: {imported} imported, {failed} failed");

    let done = dir.with_file_name(DONE_DIR);
    if let Err(e) = std::fs::rename(&dir, &done) {
        log::warn!(
            "asset logo import: cannot rename {} to {}: {e}",
            dir.display(),
            done.display()
        );
    }
}
