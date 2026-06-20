use eyre::anyhow;
use tauri::{command, AppHandle};

use super::{TranslateRequest, TranslateResponse, TranslationPrefs, TranslationService};
use crate::{logger, state::ManagerExt, util::cmd::Result};

#[command]
pub async fn translate_mods(
    mods: Vec<TranslateRequest>,
    target_language: String,
    app: AppHandle,
) -> Result<Vec<TranslateResponse>> {
    let prefs = {
        let prefs = app.lock_prefs();
        prefs.translation.clone()
    };

    let db = app.db();
    let mut results = Vec::new();
    let mut uncached_mods = Vec::new();
    let mut uncached_indices = Vec::new();

    for (i, m) in mods.iter().enumerate() {
        let cached = db.get_translation_cache(
            &m.uuid,
            &target_language,
            &m.name,
            m.description.as_deref(),
        );

        match cached {
            Ok(Some((name, desc))) => {
                results.push(Some(TranslateResponse { name, description: desc }));
            }
            _ => {
                results.push(None);
                uncached_mods.push(m.clone());
                uncached_indices.push(i);
            }
        }
    }

    if uncached_mods.is_empty() {
        return Ok(results.into_iter().flatten().collect());
    }

    let service = TranslationService::new();

    match service
        .translate_mods(&uncached_mods, &target_language, &prefs)
        .await
    {
        Ok(translated) => {
            for (idx, t) in uncached_indices.iter().zip(translated.iter()) {
                results[*idx] = Some(t.clone());
                let _ = db.save_translation_cache(
                    &mods[*idx].uuid,
                    &target_language,
                    &mods[*idx].name,
                    mods[*idx].description.as_deref(),
                    &t.name,
                    t.description.as_deref(),
                );
            }
            Ok(results.into_iter().flatten().collect())
        }
        Err(err) => {
            logger::log_webview_err("Translation failed", err, &app);
            Err(anyhow!("Translation failed").into())
        }
    }
}

#[command]
pub fn get_translation_prefs(app: AppHandle) -> TranslationPrefs {
    let prefs = app.lock_prefs();
    prefs.translation.clone()
}

#[command]
pub fn set_translation_prefs(prefs: TranslationPrefs, app: AppHandle) -> Result<()> {
    let mut current_prefs = app.lock_prefs();
    current_prefs.translation = prefs;
    app.db().save_prefs(&current_prefs)?;
    Ok(())
}
