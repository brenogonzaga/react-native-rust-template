use std::fs;
use std::path::Path;

/// JNI short-name mangling ("Resolving Native Method Names" in the JNI spec):
/// `_` becomes `_1`, then `.` becomes `_`.
fn jni_mangle(s: &str) -> String {
    s.replace('_', "_1").replace('.', "_")
}

#[test]
fn jni_entry_point_matches_the_kotlin_module() {
    let module_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../modules/rust-bridge");

    let config: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(module_dir.join("expo-module.config.json")).unwrap(),
    )
    .unwrap();
    let fqcn = config["android"]["modules"][0]
        .as_str()
        .expect("expo-module.config.json lists the Kotlin module class");
    let (package, class) = fqcn.rsplit_once('.').unwrap();

    let kotlin_path = module_dir
        .join("android/src/main/java")
        .join(fqcn.replace('.', "/"))
        .with_extension("kt");
    let kotlin = fs::read_to_string(&kotlin_path).unwrap_or_else(|e| {
        panic!(
            "expo-module.config.json names {fqcn}, but {} is unreadable: {e}",
            kotlin_path.display()
        )
    });

    let declared_package = kotlin
        .lines()
        .find_map(|l| l.trim().strip_prefix("package "))
        .map(str::trim);
    assert_eq!(declared_package, Some(package), "{}", kotlin_path.display());
    assert!(kotlin.contains(&format!("class {class}")), "{fqcn}");

    let method = kotlin
        .lines()
        .find_map(|l| l.split_once("external fun ").map(|(_, rest)| rest))
        .and_then(|rest| rest.split('(').next())
        .expect("Kotlin module declares an `external fun`");

    let symbol = format!("Java_{}_{}", jni_mangle(fqcn), jni_mangle(method));
    let lib_rs =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs")).unwrap();
    assert!(
        lib_rs.contains(&format!("fn {symbol}")),
        "\n\nKotlin calls `{fqcn}.{method}`, so the JNI entry point in \
         rust-core/src/lib.rs must be named `{symbol}`.\n"
    );
}

#[test]
fn jni_mangle_escapes_underscores_before_dots() {
    assert_eq!(jni_mangle("com.my_app.Mod"), "com_my_1app_Mod");
}
