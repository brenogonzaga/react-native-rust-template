//! Fails if a `#[ts(export)]` struct/enum carries a raw u64/i64/u128/i128/
//! usize/isize field: `serde_json` always emits the exact integer digits, and
//! JS's `JSON.parse` rounds anything past 2^53 silently. Fix: `SafeInt<T>`
//! (rust-core/src/safe_int.rs), or `#[ts(type = "string")]` / `#[ts(as = "...")]`
//! for a value with its own deliberate encoding.
//!
//! Only looks *inside* named types — a handler encoding a bare integer ad hoc
//! is caught separately, at `cargo build` time, by the `SafeForWire` bound on
//! `wire::Wire::encode`.

use quote::ToTokens;
use std::fs;
use std::path::{Path, PathBuf};

const RISKY_TYPES: [&str; 6] = ["u64", "i64", "u128", "i128", "usize", "isize"];

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|e| panic!("reading {}: {e}", dir.display())) {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

fn has_ts_attr_containing(attrs: &[syn::Attribute], needle: &str) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("ts") && attr.to_token_stream().to_string().contains(needle)
    })
}

/// Flags `u64` and friends however they're wrapped — `Option<u64>`, `Vec<i64>`,
/// etc. — by matching whole tokens in the type's printed form rather than
/// requiring an exact match, which a naive `==` would miss.
fn type_is_risky(ty: &syn::Type) -> bool {
    let printed = ty.to_token_stream().to_string();
    let tokens: Vec<&str> = printed
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|t| !t.is_empty())
        .collect();

    // `SafeInt<T>` (rust-core/src/safe_int.rs) always serializes as a JSON
    // string regardless of T — that's the type this guard's failure message
    // tells you to reach for, so seeing it wrapping the risky primitive means
    // the field is already safe, at any nesting depth (`Option<SafeInt<u64>>`
    // included).
    if tokens.contains(&"SafeInt") {
        return false;
    }

    tokens.iter().any(|token| RISKY_TYPES.contains(token))
}

fn check_fields(fields: &syn::Fields, ctx: &str, violations: &mut Vec<String>) {
    let syn::Fields::Named(named) = fields else {
        return;
    };
    for field in &named.named {
        if !type_is_risky(&field.ty) {
            continue;
        }
        if has_ts_attr_containing(&field.attrs, "\"string\"")
            || has_ts_attr_containing(&field.attrs, "as =")
        {
            continue;
        }
        let name = field
            .ident
            .as_ref()
            .map(|i| i.to_string())
            .unwrap_or_default();
        violations.push(format!("{ctx}::{name}: `{}`", field.ty.to_token_stream()));
    }
}

#[test]
fn ts_exported_types_never_carry_a_raw_64_bit_number() {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    collect_rs_files(&src_dir, &mut files);

    let mut violations = Vec::new();
    for path in &files {
        let content =
            fs::read_to_string(path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
        let file = syn::parse_file(&content)
            .unwrap_or_else(|e| panic!("failed to parse {}: {e}", path.display()));

        for item in &file.items {
            match item {
                syn::Item::Struct(s) if has_ts_attr_containing(&s.attrs, "export") => {
                    check_fields(
                        &s.fields,
                        &format!("{}::{}", path.display(), s.ident),
                        &mut violations,
                    );
                }
                syn::Item::Enum(e) if has_ts_attr_containing(&e.attrs, "export") => {
                    for variant in &e.variants {
                        check_fields(
                            &variant.fields,
                            &format!("{}::{}::{}", path.display(), e.ident, variant.ident),
                            &mut violations,
                        );
                    }
                }
                _ => {}
            }
        }
    }

    assert!(
        violations.is_empty(),
        "\n\n#[ts(export)] field(s) crossing to TS as a raw `number` but backed by a \
         64-bit+ Rust integer — JSON.parse silently rounds any value past 2^53:\n\n  {}\n\n\
         Fix: wrap the field in `SafeInt<T>` (rust-core/src/safe_int.rs) — its own \
         Serialize/TS impls carry it across as a JSON string, so there's no per-call-site \
         `.to_string()` to forget. If the value is provably always small, silence this \
         deliberately instead with #[ts(type = \"string\")] or #[ts(as = \"...\")].\n",
        violations.join("\n  ")
    );
}
