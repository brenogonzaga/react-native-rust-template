use app_core::AppService;
use once_cell::sync::Lazy;

pub static CORE_SERVICE: Lazy<AppService> = Lazy::new(AppService::new);
