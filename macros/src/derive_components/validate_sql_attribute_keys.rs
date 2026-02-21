use easy_macros::always_context;
use proc_macro2::TokenStream;
use syn::ItemStruct;

pub const DATABASE_SETUP_STRUCT_KEYS: &[&str] = &["drivers"];
pub const DATABASE_SETUP_FIELD_KEYS: &[&str] = &[];
pub const OUTPUT_STRUCT_KEYS: &[&str] = &["table", "drivers"];
pub const OUTPUT_FIELD_KEYS: &[&str] = &["field", "select", "bytes"];
pub const INSERT_STRUCT_KEYS: &[&str] = &["table", "default", "drivers"];
pub const INSERT_FIELD_KEYS: &[&str] = &["bytes"];
pub const UPDATE_STRUCT_KEYS: &[&str] = &["table", "drivers"];
pub const UPDATE_FIELD_KEYS: &[&str] = &["bytes", "maybe_update", "maybe"];
pub const TABLE_STRUCT_KEYS: &[&str] = &[
    "table_name",
    "drivers",
    "version",
    "no_version",
    "version_test",
    "unique_id",
];
pub const TABLE_FIELD_KEYS: &[&str] = &[
    "primary_key",
    "auto_increment",
    "foreign_key",
    "unique",
    "bytes",
    "default",
    "maybe_update",
    "maybe",
    "select",
];

fn canonical_easy_sql_derive_name(derive_name: &str) -> Option<&'static str> {
    match derive_name {
        "DatabaseSetup" => Some("DatabaseSetup"),
        "Output" | "OutputDebug" => Some("Output"),
        "Insert" | "InsertDebug" => Some("Insert"),
        "Update" | "UpdateDebug" => Some("Update"),
        "Table" | "TableDebug" => Some("Table"),
        _ => None,
    }
}

fn supported_keys_for_derive(
    derive_name: &str,
) -> Option<(&'static [&'static str], &'static [&'static str])> {
    match canonical_easy_sql_derive_name(derive_name)? {
        "DatabaseSetup" => Some((DATABASE_SETUP_STRUCT_KEYS, DATABASE_SETUP_FIELD_KEYS)),
        "Output" => Some((OUTPUT_STRUCT_KEYS, OUTPUT_FIELD_KEYS)),
        "Insert" => Some((INSERT_STRUCT_KEYS, INSERT_FIELD_KEYS)),
        "Update" => Some((UPDATE_STRUCT_KEYS, UPDATE_FIELD_KEYS)),
        "Table" => Some((TABLE_STRUCT_KEYS, TABLE_FIELD_KEYS)),
        _ => None,
    }
}

fn push_unique_key(target: &mut Vec<String>, key: &str) {
    if target.iter().any(|existing| existing == key) {
        return;
    }
    target.push(key.to_string());
}

fn extend_unique_keys(target: &mut Vec<String>, keys: &[&str]) {
    for key in keys {
        push_unique_key(target, key);
    }
}

fn active_easy_sql_derives(item: &ItemStruct) -> Vec<&'static str> {
    let mut derives = Vec::new();

    for attr in &item.attrs {
        if !attr.path().is_ident("derive") {
            continue;
        }

        let _ = attr.parse_nested_meta(|meta| {
            if let Some(ident) = meta.path.segments.last().map(|segment| &segment.ident)
                && let Some(canonical) = canonical_easy_sql_derive_name(&ident.to_string())
                && !derives.contains(&canonical)
            {
                derives.push(canonical);
            }
            Ok(())
        });
    }

    derives
}

fn effective_supported_keys(
    item: &ItemStruct,
    derive_name: &str,
    struct_supported_fallback: &[&str],
    field_supported_fallback: &[&str],
) -> (Vec<String>, Vec<String>) {
    let mut effective_struct = Vec::<String>::new();
    let mut effective_field = Vec::<String>::new();

    if let Some((struct_keys, field_keys)) = supported_keys_for_derive(derive_name) {
        extend_unique_keys(&mut effective_struct, struct_keys);
        extend_unique_keys(&mut effective_field, field_keys);
    }

    extend_unique_keys(&mut effective_struct, struct_supported_fallback);
    extend_unique_keys(&mut effective_field, field_supported_fallback);

    let current_derive = canonical_easy_sql_derive_name(derive_name);
    for active_derive in active_easy_sql_derives(item) {
        if Some(active_derive) == current_derive {
            continue;
        }

        if let Some((struct_keys, field_keys)) = supported_keys_for_derive(active_derive) {
            extend_unique_keys(&mut effective_struct, struct_keys);
            extend_unique_keys(&mut effective_field, field_keys);
        }
    }

    (effective_struct, effective_field)
}

fn push_combined_error(combined_error: &mut Option<syn::Error>, error: syn::Error) {
    if let Some(existing) = combined_error {
        existing.combine(error);
    } else {
        *combined_error = Some(error);
    }
}

fn levenshtein(left: &str, right: &str) -> usize {
    if left == right {
        return 0;
    }
    if left.is_empty() {
        return right.chars().count();
    }
    if right.is_empty() {
        return left.chars().count();
    }

    let right_chars = right.chars().collect::<Vec<_>>();
    let mut prev_row = (0..=right_chars.len()).collect::<Vec<_>>();

    for (left_index, left_char) in left.chars().enumerate() {
        let mut current_row = Vec::with_capacity(right_chars.len() + 1);
        current_row.push(left_index + 1);

        for (right_index, right_char) in right_chars.iter().enumerate() {
            let substitution_cost = usize::from(left_char != *right_char);
            let insertion = current_row[right_index] + 1;
            let deletion = prev_row[right_index + 1] + 1;
            let substitution = prev_row[right_index] + substitution_cost;
            current_row.push(insertion.min(deletion).min(substitution));
        }

        prev_row = current_row;
    }

    prev_row[right_chars.len()]
}

fn best_key_suggestion<'a>(unknown: &str, supported: &'a [&str]) -> Option<&'a str> {
    if supported.is_empty() {
        return None;
    }

    let unknown_lower = unknown.to_lowercase();

    let mut best: Option<(&str, usize)> = None;
    let mut has_tie = false;
    for candidate in supported {
        let distance = levenshtein(&unknown_lower, &candidate.to_lowercase());
        match best {
            None => best = Some((candidate, distance)),
            Some((_, best_distance)) if distance < best_distance => {
                best = Some((candidate, distance));
                has_tie = false;
            }
            Some((_, best_distance)) if distance == best_distance => {
                has_tie = true;
            }
            _ => {}
        }
    }

    let (candidate, distance) = best?;
    if has_tie || distance > 2 {
        return None;
    }

    Some(candidate)
}

fn is_additionally_accepted_sql_key(derive_name: &str, context_name: &str, key_name: &str) -> bool {
    if context_name != "struct" || key_name != "default" {
        return false;
    }

    matches!(
        canonical_easy_sql_derive_name(derive_name),
        Some("Output") | Some("Update")
    )
}

fn scan_sql_attrs_for_unknown_keys(
    attrs: &[syn::Attribute],
    derive_name: &str,
    context_name: &str,
    supported: &[&str],
    combined_error: &mut Option<syn::Error>,
) {
    for attr in attrs {
        if !attr.path().is_ident("sql") {
            continue;
        }

        let parse_result = attr.parse_nested_meta(|meta| {
            let Some(ident) = meta.path.get_ident() else {
                return Ok(());
            };

            let key_name = ident.to_string();
            if supported
                .iter()
                .any(|supported_key| *supported_key == key_name)
            {
                return Ok(());
            }

            if is_additionally_accepted_sql_key(derive_name, context_name, &key_name) {
                return Ok(());
            }

            let suggestion = best_key_suggestion(&key_name, supported);
            let supported_attrs = if supported.is_empty() {
                String::from("(none)")
            } else {
                supported.join(",")
            };

            let mut message = format!("Unknown sql {} attribute `{}`", context_name, key_name);
            if let Some(suggested) = suggestion {
                message.push_str(&format!(" Did you mean `{}`?", suggested));
            }
            message.push_str(&format!(
                " Supported {} attributes for {}: {}.",
                context_name, derive_name, supported_attrs
            ));

            push_combined_error(
                combined_error,
                syn::Error::new_spanned(meta.path.clone(), message),
            );
            Ok(())
        });

        if parse_result.is_err() {
            continue;
        }
    }
}

#[always_context]
pub fn validate_sql_attribute_keys(
    item: &ItemStruct,
    derive_name: &str,
    struct_supported: &[&str],
    field_supported: &[&str],
) -> Option<TokenStream> {
    let mut combined_error = None;

    let (effective_struct_supported, effective_field_supported) =
        effective_supported_keys(item, derive_name, struct_supported, field_supported);
    let effective_struct_supported_refs = effective_struct_supported
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let effective_field_supported_refs = effective_field_supported
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();

    scan_sql_attrs_for_unknown_keys(
        &item.attrs,
        derive_name,
        "struct",
        &effective_struct_supported_refs,
        &mut combined_error,
    );

    for field in item.fields.iter() {
        scan_sql_attrs_for_unknown_keys(
            &field.attrs,
            derive_name,
            "field",
            &effective_field_supported_refs,
            &mut combined_error,
        );
    }

    combined_error.map(|error| error.to_compile_error())
}
