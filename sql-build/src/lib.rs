use std::{
    collections::{HashMap, hash_map::Entry},
    io::Write,
    path::Path,
};

use convert_case::{Case, Casing};
use easy_macros::{
    anyhow::{self, Context},
    helpers::context,
    macros::{all_syntax_cases, always_context, get_attributes},
    proc_macro2::LineColumn,
    quote::{ToTokens, quote},
    syn::{
        self, ItemFn, ItemImpl, ItemTrait, LitInt, LitStr, Macro, Meta, punctuated::Punctuated,
        spanned::Spanned,
    },
};
use sql_compilation_data::{CompilationData, TableData, TableDataVersion};
#[derive(Debug)]
struct SearchData {
    ///When parsing rust files
    errors_found: bool,
    found: bool,
    ///Where to add `#[sql_convenience]`
    updates: Vec<LineColumn>,

    //SqlTable handling
    created_unique_ids: Vec<(String, LineColumn)>,
    compilation_data: CompilationData,
    found_existing_tables_ids: Vec<String>,
    //Use also created_unique_ids to check if tables were updated
    tables_updated: bool,
    ///Will be added to logs
    /// Also add to logs when there were no errors
    unsorted_errors: Vec<anyhow::Error>,
    file_matched_errors: Vec<(String, Vec<anyhow::Error>)>,
}

impl SearchData {
    fn new(compilation_data: CompilationData) -> Self {
        SearchData {
            errors_found: false,
            found: false,
            updates: Vec::new(),
            created_unique_ids: Vec::new(),
            compilation_data,
            found_existing_tables_ids: Vec::new(),
            tables_updated: false,
            unsorted_errors: Vec::new(),
            file_matched_errors: Vec::new(),
        }
    }
}

all_syntax_cases! {
    setup=>{
        generated_fn_prefix:"macro_search",
        additional_input_type:&mut SearchData
    }
    default_cases=>{
        fn struct_table_handle_wrapper(item: &mut syn::ItemStruct, context_info: &mut SearchData);

        fn macro_check(item: &mut Macro, context_info: &mut SearchData);
    }
    special_cases=>{
        fn item_fn_check(item: &mut ItemFn, context_info: &mut SearchData);
        fn trait_check(item: &mut ItemTrait, context_info: &mut SearchData);
        fn impl_check(item: &mut ItemImpl, context_info: &mut SearchData);
    }
}

struct DeriveInsides {
    list: Punctuated<syn::Path, syn::Token![,]>,
}

impl syn::parse::Parse for DeriveInsides {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let list = Punctuated::<syn::Path, syn::Token![,]>::parse_terminated(input)?;
        Ok(DeriveInsides { list })
    }
}

///Table handling
#[always_context]
fn struct_table_handle(
    item: &mut syn::ItemStruct,
    context_info: &mut SearchData,
) -> anyhow::Result<()> {
    if let Some(attr) = get_attributes!(item, #[derive(__unknown__)])
        .into_iter()
        .next()
    {
        let parsed = match syn::parse2::<DeriveInsides>(attr) {
            Ok(parsed) => parsed.list,
            Err(_) => {
                //Ignore invalid attributes, error should be shown by derive macro
                return Ok(());
            }
        };
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
            return Ok(());
        }
    } else {
        //No Sql Table derive
        return Ok(());
    }
    //Check is unique_id present
    let mut unique_id = None;
    for attr_data in get_attributes!(item, #[sql(unique_id = __unknown__)]) {
        if unique_id.is_some() {
            //Ignore multiple unique_id attributes, error should be shown by derive macro
            return Ok(());
        }
        let lit_str: LitStr = syn::parse2(attr_data.clone())?;
        unique_id = Some(lit_str.value());
    }
    //Unique Id
    let newly_created = if unique_id.is_none() {
        //Create unique_id
        let generated = context_info.compilation_data.generate_unique_id();
        context_info
            .created_unique_ids
            .push((generated.clone(), item.struct_token.span().start()));

        unique_id = Some(generated);
        true
    } else {
        false
    };

    let unique_id = unique_id.unwrap();

    context_info
        .found_existing_tables_ids
        .push(unique_id.clone());

    //Check if table version has changed
    let mut version = None;
    for attr_data in get_attributes!(item, #[sql(version = __unknown__)]) {
        let lit_int: LitInt = match syn::parse2(attr_data) {
            Ok(lit_int) => lit_int,
            Err(_) => {
                //Ignore invalid attributes, error should be shown by derive macro
                return Ok(());
            }
        };
        version = match lit_int.base10_parse::<u64>() {
            Ok(o) => Some(o),
            Err(_) => {
                //Ignore invalid attributes, error should be shown by derive macro
                return Ok(());
            }
        };
    }

    //Version attribute should exist. if it doesn't error by derive macro should be shown
    #[no_context]
    let version = version.context("Version attribute should exist")?;

    match &item.fields {
        syn::Fields::Named(_) => {}
        _ => {
            //Ignore unnamed and unit structs, error should be shown by derive macro, leave debug info
            anyhow::bail!("non named fields, struct: {}", item.to_token_stream());
        }
    }

    let mut table_name = item.ident.to_string().to_case(Case::Snake);
    //Check if table_name was set manually
    for attr_data in get_attributes!(item, #[sql(table_name = "__unknown__")]) {
        let lit_str: LitStr = syn::parse2(attr_data.clone())?;
        table_name = lit_str.value();
        break;
    }

    //Generate table version data
    let version_data = TableDataVersion::from_struct(item, table_name.clone())?;

    //Migration check if data exists before
    if !newly_created {
        context_info
            .compilation_data
            .generate_migrations(&unique_id, &version_data, version, &quote! {})
            .with_context(|| format!("Compilation data: {:?}", context_info.compilation_data))?;
    }

    match context_info.compilation_data.tables.entry(unique_id) {
        Entry::Occupied(occupied_entry) => {
            let table_data = occupied_entry.into_mut();
            //Ignore when current version is smaller than the latest version (Error by derive macro)
            if table_data.latest_version < version {
                table_data.saved_versions.insert(version, version_data);

                table_data.latest_version = version;
                context_info.tables_updated = true;
            }
        }
        Entry::Vacant(vacant_entry) => {
            let mut saved_versions = HashMap::new();
            saved_versions.insert(version, version_data);

            let table_data = TableData {
                latest_version: version,
                saved_versions,
            };

            vacant_entry.insert(table_data);
        }
    }

    Ok(())
}

fn struct_table_handle_wrapper(item: &mut syn::ItemStruct, context_info: &mut SearchData) {
    match struct_table_handle(item, context_info) {
        Ok(_) => {}
        Err(err) => {
            context_info.unsorted_errors.push(err);
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
    for item in item.items.iter_mut() {
        macro_search_trait_item_handle(item, context_info);
    }

    if context_info.found && !has_sql_convenience(&item.attrs) {
        context_info.updates.push(item.span().start());
        context_info.found = false;
    }
}

fn impl_check(item: &mut ItemImpl, context_info: &mut SearchData) {
    for item in item.items.iter_mut() {
        macro_search_impl_item_handle(item, context_info);
    }

    if context_info.found && !has_sql_convenience(&item.attrs) {
        context_info.updates.push(item.span().start());
        context_info.found = false;
    }
}

fn item_fn_check(item: &mut ItemFn, context_info: &mut SearchData) {
    macro_search_block_handle(&mut item.block, context_info);

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
fn handle_file(file_path: impl AsRef<Path>, search_data: &mut SearchData) -> anyhow::Result<()> {
    let file_path = file_path.as_ref();
    // Check if the file is a rust file
    match file_path.extension() {
        Some(ext) if ext == "rs" => {}
        _ => return Ok(()),
    }

    // Read the file
    let mut contents = std::fs::read_to_string(file_path)?;
    //Operate on syn::File
    let file = match syn::parse_file(&contents) {
        Ok(file) => file,
        Err(_) => {
            //Don't delete tables if at least one file has errors
            search_data.errors_found = true;
            //Ignore files with errors
            return Ok(());
        }
    };

    for mut item in file.items.into_iter() {
        search_data.found = false;
        handle_item(
            #[context(tokens)]
            &mut item,
            search_data,
        )?;
    }

    // Create #[sql_convenience] in the file (if needed)
    if !search_data.updates.is_empty() {
        let mut updates = search_data.updates.drain(..).collect::<Vec<_>>();
        //Sort our lines (reverse order)
        updates.sort_by(|a, b| b.line.cmp(&a.line));

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

    //Create unique ids in the file (if needed)
    if !search_data.created_unique_ids.is_empty() {
        search_data.tables_updated = true;

        let mut updates = search_data.created_unique_ids.drain(..).collect::<Vec<_>>();
        //Sort our lines (reverse order)
        updates.sort_by(|a, b| b.1.line.cmp(&a.1.line));

        //Uses span position info to add #[sql(unique_id = ...)] to every item on the list
        for (unique_id, start_pos) in updates.into_iter() {
            //1 indexed
            let line = start_pos.line;
            //Find position based on line
            let line_bytes_end = line_pos(&contents, line - 1)?;

            contents.insert_str(
                line_bytes_end,
                &format!("#[sql(unique_id = \"{}\")]\r\n", unique_id),
            );
        }

        let mut file = std::fs::File::create(file_path).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
    }

    //File match errors
    if !search_data.unsorted_errors.is_empty() {
        search_data.file_matched_errors.push((
            file_path.display().to_string(),
            search_data.unsorted_errors.drain(..).collect(),
        ));
    }

    Ok(())
}

#[always_context]
fn handle_dir(
    dir: impl AsRef<Path>,
    ignore_list: &[regex::Regex],
    base_path_len_bytes: usize,
    search_data: &mut SearchData,
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
            handle_file(&entry_path, search_data)?;
        } else if file_type.is_dir() {
            // If the file is a directory, call this function recursively
            handle_dir(&entry_path, ignore_list, base_path_len_bytes, search_data)?;
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

    let mut search_data = SearchData::new(CompilationData::load()?);

    handle_dir(&src_dir, ignore_list, base_path_len_bytes, &mut search_data)?;

    //Write into log file (if needed)
    if !search_data.file_matched_errors.is_empty() {
        let log_folder = current_dir.join("easy_sql_logs");
        if !log_folder.exists() {
            std::fs::create_dir_all(&log_folder)?;
        }
        let current_date = chrono::Utc::now();
        let log_file = log_folder.join(format!("{}.txt", current_date.format("%Y-%m-%d")));

        let errors = search_data
            .file_matched_errors
            .iter()
            .map(|(file_path, errors)| {
                let mut error_str =
                    format!("==========\r\nFile: {}\r\n==========\r\n\r\n", file_path);
                for err in errors.iter() {
                    error_str.push_str(&format!("{:?}\r\n\r\n", err));
                }
                error_str
            })
            .collect::<Vec<_>>()
            .join("\n");

        let log_header = format!(
            "==================\r\n[[[{} - Build Log]]]\r\n==================\r\n\r\n{}\r\n\r\n",
            current_date.format("%H:%M:%S"),
            errors
        );
        let mut log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)?;
        log_file.write_all(log_header.as_bytes())?;
    }

    //Remove deleted tables
    // If no errors when parsing rust files were found
    if !search_data.errors_found
        && search_data.compilation_data.tables.len() != search_data.found_existing_tables_ids.len()
    {
        search_data.tables_updated = true;

        search_data.compilation_data.tables.retain(|key, _| {
            if search_data.found_existing_tables_ids.contains(key) {
                return true;
            }
            //Table was deleted
            false
        });
    }

    //Update compilation data (if needed)
    search_data.compilation_data.save()?;

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
