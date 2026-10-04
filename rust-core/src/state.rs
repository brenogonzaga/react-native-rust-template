use app_core::AppService;
use std::sync::LazyLock;

pub static CORE_SERVICE: LazyLock<AppService> = LazyLock::new(AppService::new);
