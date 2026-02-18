use easy_macros::always_context;
use proc_macro2::TokenStream;
use syn::ItemStruct;

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

    scan_sql_attrs_for_unknown_keys(
        &item.attrs,
        derive_name,
        "struct",
        struct_supported,
        &mut combined_error,
    );

    for field in item.fields.iter() {
        scan_sql_attrs_for_unknown_keys(
            &field.attrs,
            derive_name,
            "field",
            field_supported,
            &mut combined_error,
        );
    }

    combined_error.map(|error| error.to_compile_error())
}
