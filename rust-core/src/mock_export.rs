use app_core::{AppVersion, User};
use serde::Serialize;
use ts_rs::TS;

/// ⚠️ ARCHITECTURE NOTE FOR DEVELOPERS:
/// For any struct (e.g., User, AppVersion) to be added to this mock registry and auto-exported
/// to TypeScript, the source struct MUST implement `#[derive(ts_rs::TS)]`.
///
/// If the struct belongs to a 3rd-party closed crate without `#[derive(TS)]`, you can either:
/// 1. Use `#[ts(type = "string")]` or `#[ts(as = "...")]` on the corresponding field.
/// 2. Map the struct using a local transparent wrapper with `#[serde(transparent)]`.
#[derive(Serialize, TS)]
#[ts(export)]
#[allow(dead_code)]
pub struct GlobalTypesMock {
    pub user: User,
    pub app_version: AppVersion,
}
