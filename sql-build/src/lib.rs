use std::{collections::hash_map::Entry, io::Write, path::Path};

use easy_macros::{
    anyhow::{self, Context},
    helpers::context,
    macros::{all_syntax_cases, always_context, get_attributes},
    proc_macro2::LineColumn,
    quote::ToTokens,
    syn::{
        self, ItemFn, ItemImpl, ItemTrait, LitInt, LitStr, Macro, Meta, punctuated::Punctuated,
        spanned::Spanned,
    },
};
use sql_compilation_data::{CompilationData, TableData, TableDataVersion, TableField};
#[derive(Debug, Default)]
struct SearchData {
    found: bool,
    ///Where to add `#[sql_convenience]`
    updates: Vec<LineColumn>,

    //SqlTable handling
    created_unique_ids: Vec<(String, LineColumn)>,
    compilation_data: CompilationData,
    found_existing_tables_ids: Vec<String>,
    //Use also created_unique_ids to check if tables were updated
    tables_updated: bool,
}

all_syntax_cases! {
    setup=>{
        generated_fn_prefix:"macro_search",
        additional_input_type:&mut SearchData
    }
    default_cases=>{
        fn struct_table_handle(item: &mut syn::ItemStruct, context_info: &mut SearchData);
        #[after_system]
        fn item_fn_check(item: &mut ItemFn, context_info: &mut SearchData);
        #[after_system]
        fn trait_check(item: &mut ItemTrait, context_info: &mut SearchData);
        #[after_system]
        fn impl_check(item: &mut ItemImpl, context_info: &mut SearchData);
        fn macro_check(item: &mut Macro, context_info: &mut SearchData);
    }
    special_cases=>{}
}

fn generate_table_data_from_struct(item: &syn::ItemStruct, table_name: String) -> TableDataVersion {
    let fields = match item.fields {
        syn::Fields::Named(fields_named) => fields_named.named,
        _ => {
            unreachable!(
                "non named field type should be handled before `generate_table_data_from_struct` is called"
            )
        }
    };

    let fields_converted=Vec::new();

    for field in fields.iter(){
        let name=field.ident.as_ref().unwrap().to_string();

        fields_converted.push(TableField{
            name,
            sql_type: ,
            is_primary_key: todo!(),
            has_default: todo!(),
        });
    }

    TableDataVersion { table_name, fields: fields_converted }
}

///Table handling
fn struct_table_handle(item: &mut syn::ItemStruct, context_info: &mut SearchData) {
    if let Some(attr) = get_attributes!(item, #[derive(__unknown__)])
        .into_iter()
        .next()
    {
        let parsed = Punctuated::<syn::Path, syn::Token![,]>::parse_terminated(&attr).unwrap();
        let mut is_sql_table = false;
        for path in parsed.iter() {
            let path_str = path
                .to_token_stream()
                .to_string()
                .replace(|c: char| c.is_whitespace(), "");
            match path_str.as_str() {
                "SqlTable" | "easy_lib::sql::SqlTable" => {
                    is_sql_table = true;
                }
                _ => {}
            }
        }
        if !is_sql_table {
            //No Sql Table derive
            return;
        }
    } else {
        //No Sql Table derive
        return;
    }
    //Check is unique_id present
    let mut unique_id = None;
    for attr_data in get_attributes!(item, #[sql(unique_id = "__unknown__")]) {
        if unique_id.is_some() {
            //Ignore multiple unique_id attributes, error should be shown by derive macro
            return;
        }
        let lit_str: LitStr = match syn::parse2(attr_data) {
            Ok(lit_str) => lit_str,
            Err(_) => {
                //Ignore invalid attributes, error should be shown by derive macro
                return;
            }
        };
        unique_id = Some(lit_str.value());
    }
    //Unique Id
    if let Some(unique_id) = unique_id {
        //Table wasn't deleted
        context_info.found_existing_tables_ids.push(unique_id);
    } else {
        //Create unique_id
        let generated = context_info.compilation_data.generate_unique_id();
        context_info
            .created_unique_ids
            .push((generated.clone(), item.span().start()));

        unique_id = Some(generated);
    }

    let unique_id = unique_id.unwrap();

    //Check if table version has changed
    let mut version = None;
    for attr_data in get_attributes!(item, #[sql(version = "__unknown__")]) {
        let lit_int: LitInt = match syn::parse2(attr_data) {
            Ok(lit_int) => lit_int,
            Err(_) => {
                //Ignore invalid attributes, error should be shown by derive macro
                return;
            }
        };
        version = match lit_int.base10_parse::<u64>() {
            Ok(o) => Some(o),
            Err(_) => {
                //Ignore invalid attributes, error should be shown by derive macro
                return;
            }
        };
    }

    //Version attribute should exist. if it doesn't error by derive macro should be shown
    let version = match version {
        Some(o) => o,
        None => {
            return;
        }
    };

    match item.fields {
        syn::Fields::Named(fields_named) => {}
        _ => {
            //Ignore unnamed and unit structs, error should be shown by derive macro
            return;
        }
    }

    let table_name = item.ident.to_string().to_case(Case::Snake);
    //Check if table_name was set manually
    for attr_data in get_attributes!(item, #[sql(table_name = "__unknown__")]) {
        let lit_str: LitStr = match syn::parse2(attr_data) {
            Ok(lit_str) => lit_str,
            Err(_) => {
                //Ignore invalid attributes, error should be shown by derive macro
                return;
            }
        };
        table_name = lit_str.value();
        break;
    }

    match context_info.compilation_data.tables.entry(unique_id) {
        Entry::Occupied(occupied_entry) => {
            let table_data = occupied_entry.into_mut();
            //Ignore when current version is smaller than the latest version (Error by derive macro)
            if table_data.latest_version < version {
                table_data
                    .saved_versions
                    .push(generate_table_data_from_struct(item, table_name));

                table_data.latest_version = version;
                context_info.tables_updated = true;
            }
        }
        Entry::Vacant(vacant_entry) => {
            let table_version = generate_table_data_from_struct(item, table_name);

            let table_data = TableData {
                latest_version: version,
                saved_versions: vec![table_version],
            };

            vacant_entry.insert(table_data);
        }
    }
}

fn macro_check(item: &mut Macro, context_info: &mut SearchData) {
    let path = item.path.to_token_stream().to_string();
    match path.as_str() {
        "sql" | "sql_where" | "easy_lib::sql::sql" | "easy_lib::sql::sql_where" => {
            context_info.found = true;
        }
        _ => {}
    }
}

fn trait_check(item: &mut ItemTrait, context_info: &mut SearchData) {
    if context_info.found && !has_sql_convenience(&item.attrs) {
        context_info.updates.push(item.span().start());
        context_info.found = false;
    }
}

fn impl_check(item: &mut ItemImpl, context_info: &mut SearchData) {
    if context_info.found && !has_sql_convenience(&item.attrs) {
        context_info.updates.push(item.span().start());
        context_info.found = false;
    }
}

fn item_fn_check(item: &mut ItemFn, context_info: &mut SearchData) {
    if context_info.found && !has_sql_convenience(&item.attrs) {
        context_info.updates.push(item.span().start());
        context_info.found = false;
    }
}

fn has_sql_convenience(attrs: &[syn::Attribute]) -> bool {
    for attr in attrs {
        if let Meta::Path(path) = &attr.meta {
            let path_str = path
                .to_token_stream()
                .to_string()
                .replace(|c: char| c.is_whitespace(), "");
            if let "sql_convenience" | "easy_lib::sql::sql_convenience" = path_str.as_str() {
                return true;
            }
        }
    }
    false
}

#[always_context]
fn handle_item(item: &mut syn::Item, updates: &mut SearchData) -> anyhow::Result<()> {
    macro_search_item_handle(item, updates);
    Ok(())
}
/// # Inputs
/// `line` - 0 indexed
#[always_context]
fn line_pos(haystack: &str, line: usize) -> anyhow::Result<usize> {
    let mut regex_str = "^".to_string();
    for _ in 0..line {
        regex_str.push_str(r".*((\r\n)|\r|\n)");
    }
    let regex = regex::Regex::new(&regex_str)?;

    let found = regex
        .find_at(haystack, 0)
        .with_context(context!("Finding line failed! | Regex: {:?}", regex))?;

    Ok(found.end())
}

#[always_context]
fn handle_file(file_path: impl AsRef<Path>) -> anyhow::Result<()> {
    let file_path = file_path.as_ref();
    // Check if the file is a rust file
    match file_path.extension() {
        Some(ext) if ext == "rs" => {}
        _ => return Ok(()),
    }

    // Read the file
    let mut contents = std::fs::read_to_string(file_path)?;
    //Operate on syn::File
    let mut search_data: SearchData = SearchData::default();
    let file = match syn::parse_file(&contents) {
        Ok(file) => file,
        Err(_) => {
            //Ignore files with errors
            return Ok(());
        }
    };

    for mut item in file.items.into_iter() {
        handle_item(
            #[context(tokens)]
            &mut item,
            &mut search_data,
        )?;
    }

    // Update the file (if needed)
    if !search_data.updates.is_empty() {
        let mut updates = search_data.updates;
        //Sort our lines and reverse them
        updates.sort_by(|a, b| a.line.cmp(&b.line));
        updates.reverse();

        //Uses span position info to add #[sql_convenience] to every item on the list
        for start_pos in updates.into_iter() {
            //1 indexed
            let line = start_pos.line;
            //Find position based on line
            let line_bytes_end = line_pos(&contents, line - 1)?;

            contents.insert_str(line_bytes_end, "#[sql_convenience]\r\n");
        }

        let mut file = std::fs::File::create(file_path).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
    }

    Ok(())
}

#[always_context]
fn handle_dir(
    dir: impl AsRef<Path>,
    ignore_list: &[regex::Regex],
    base_path_len_bytes: usize,
) -> anyhow::Result<()> {
    // Get all files in the src directory
    let files = std::fs::read_dir(dir.as_ref())?;
    // Iterate over all files
    'entries: for entry in files {
        #[no_context_inputs]
        let entry = entry?;

        // Get the file path
        let entry_path = entry.path();

        //Ignore list check
        for r in ignore_list.iter() {
            let path_str = entry_path.display().to_string();

            if r.is_match(&path_str[base_path_len_bytes..]) {
                // Ignore this entry
                continue 'entries;
            }
        }

        let file_type = entry.file_type()?;
        if file_type.is_file() {
            handle_file(&entry_path)?;
        } else if file_type.is_dir() {
            // If the file is a directory, call this function recursively
            handle_dir(&entry_path, ignore_list, base_path_len_bytes)?;
        }
    }

    Ok(())
}

#[always_context]
/// Build function that adds `#[always_context]` attribute to every function with `anyhow::Result` return type and every `trait` and `impl` block.
///
/// To every rust file in `src` directory.
///
/// # Arguments
///
/// `ignore_list` - A list of regex patterns to ignore. The patterns are used on the file path. Path is ignored if match found.
///
fn build_result(ignore_list: &[regex::Regex]) -> anyhow::Result<()> {
    // Get the current directory
    let current_dir = std::env::current_dir()?;
    let base_path_len_bytes = current_dir.display().to_string().len();
    // Get the src directory
    let src_dir = current_dir.join("src");

    handle_dir(&src_dir, ignore_list, base_path_len_bytes)?;

    Ok(())
}
/// Build function that adds `#[always_context]` attribute to every function with `anyhow::Result` return type and every `trait` and `impl` block.
///
/// To every rust file in `src` directory.
///
/// Panics on error. Use `build_result()` for error handling.
///
/// # Arguments
///
/// `ignore_list` - A list of regex patterns to ignore. The patterns are used on the file path. Path is ignored if match found.
///
pub fn build(ignore_list: &[regex::Regex]) {
    if let Err(err) = build_result(ignore_list) {
        panic!(
            "Always Context Build Error: {}\r\n\r\nDebug Info:\r\n\r\n{:?}",
            err, err
        );
    }
}
