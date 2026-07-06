use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
    str::FromStr,
};
#[cfg(feature = "migrations")]
use std::borrow::Cow;

use anyhow::{self, Context};
use quote::ToTokens;
#[cfg(feature = "migrations")]
use {
    easy_macros::TokensBuilder,
    proc_macro2::{Span, TokenStream},
    quote::quote,
};

use easy_macros::{
    always_context, get_attributes, has_attributes, token_stream_to_consistent_string,
};
use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TableField {
    pub name: String,
    #[serde(default)]
    pub ty_to_bytes: bool,
    pub field_type: String,
    ///Tokens converted to_string()
    pub default: Option<String>,
    pub is_unique: bool,
}

#[cfg(feature = "migrations")]
impl TableField {
    /// Returns the normalized persisted storage type used by migration comparisons.
    /// Reason: `#[sql(bytes)]` fields may legitimately change Rust wrapper types while still storing the same nullable blob column, so migration generation must compare the storage contract instead of the wrapper name.
    fn migration_storage_type(&self) -> Cow<'_, str> {
        // Normalize bytes-backed fields to the blob type they actually persist while preserving optionality.
        if self.ty_to_bytes {
            if self.field_type.starts_with("Option<") && self.field_type.ends_with('>') {
                return Cow::Borrowed("Option<Vec<u8>>");
            }

            return Cow::Borrowed("Vec<u8>");
        }

        // Keep non-bytes fields on their declared type so true schema changes still fail migration generation.
        Cow::Borrowed(&self.field_type)
    }

    /// Returns whether two field definitions keep the same persisted storage contract.
    /// Reason: migration generation should reject only real schema-shape changes, not bytes-wrapper differences that still map to the same SQL blob representation.
    fn is_migration_storage_compatible_with(&self, other: &Self) -> bool {
        self.migration_storage_type() == other.migration_storage_type()
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct TableDataVersion {
    pub table_name: String,
    pub fields: Vec<TableField>,
    pub primary_keys: Vec<String>,
    pub auto_increment: bool,
    ///key - table (struct) name
    ///value - current field name
    #[serde(serialize_with = "ordered_map")]
    pub foreign_keys: HashMap<String, Vec<String>>,
}

fn ordered_map<S, K: Ord + Serialize, V: Serialize>(
    value: &HashMap<K, V>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}

impl TableDataVersion {
    pub fn from_struct(item: &syn::ItemStruct, table_name: String) -> anyhow::Result<Self> {
        let fields = match &item.fields {
            syn::Fields::Named(fields_named) => &fields_named.named,
            _ => {
                anyhow::bail!(
                    "non named field type should be handled before `generate_table_data_from_struct` is called"
                )
            }
        };

        let mut fields_converted = Vec::new();
        let mut primary_keys = Vec::new();
        let mut foreign_keys = HashMap::new();

        let mut auto_increment = false;

        for field in fields.iter() {
            let name = field.ident.as_ref().unwrap().to_string();

            //Auto Increment Check
            if has_attributes!(field, #[sql(auto_increment)]) {
                if auto_increment {
                    anyhow::bail!("Auto increment is only supported for single primary key");
                }
                auto_increment = true;
            }

            let ty_to_bytes = has_attributes!(field, #[sql(bytes)]);

            let mut default = None;
            for default_value in get_attributes!(field, #[sql(default = __unknown__)]) {
                if default.is_some() {
                    anyhow::bail!("Only one #[sql(default = ...)] attribute is allowed per field");
                }

                let default_expr: syn::Expr = syn::parse2(default_value.clone())
                    .context("Expected #[sql(default = ...)] to contain a valid Rust expression")?;

                default = Some(token_stream_to_consistent_string(
                    default_expr.to_token_stream(),
                ));
            }

            for foreign_key in get_attributes!(field, #[sql(foreign_key = __unknown__)])
                .into_iter()
                .map(token_stream_to_consistent_string)
            {
                let fields: &mut Vec<String> = foreign_keys
                    .entry(foreign_key)
                    .or_insert(Default::default());
                fields.push(name.clone());
            }

            if has_attributes!(field, #[sql(primary_key)]) {
                primary_keys.push(name.clone());
            }

            let is_unique = has_attributes!(field, #[sql(unique)]);

            fields_converted.push(TableField {
                name,
                field_type: token_stream_to_consistent_string(field.ty.to_token_stream()),
                default,
                is_unique,
                ty_to_bytes,
            });
        }

        Ok(TableDataVersion {
            table_name,
            fields: fields_converted,
            foreign_keys,
            primary_keys,
            auto_increment,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TableData {
    #[serde(serialize_with = "ordered_map")]
    pub saved_versions: HashMap<i64, TableDataVersion>,
    pub latest_version: i64,
}
#[cfg(feature = "check_duplicate_table_names")]
#[derive(Debug, Serialize, Deserialize)]
pub struct TableNameData {
    pub filename: String,
    pub struct_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompilationData {
    ///Key - table id, generated by build macro, put in #[sql(unique_id = "...")] attribute on struct
    #[serde(serialize_with = "ordered_map")]
    pub tables: HashMap<String, TableData>,
    #[cfg(feature = "check_duplicate_table_names")]
    #[serde(serialize_with = "ordered_map")]
    #[serde(default)]
    ///Key - table name
    pub used_table_names: HashMap<String, Vec<TableNameData>>,
    #[serde(default)]
    pub default_drivers: Vec<String>,
}
#[always_context]
impl CompilationData {
    pub fn data_location() -> anyhow::Result<PathBuf> {
        let manifest_dir_str = std::env::var("CARGO_MANIFEST_DIR")?;
        let current_dir = PathBuf::from_str(&manifest_dir_str)?;

        Ok(current_dir.join("easy_sql.ron"))
    }

    #[cfg(feature = "build")]
    pub fn load(
        default_drivers: Vec<String>,
        default_drivers_update: bool,
    ) -> anyhow::Result<CompilationData> {
        let data_path = Self::data_location()?;

        let data: CompilationData = {
            if !data_path.exists() {
                CompilationData {
                    tables: HashMap::new(),
                    #[cfg(feature = "check_duplicate_table_names")]
                    used_table_names: HashMap::new(),
                    default_drivers,
                }
            } else {
                let data = std::fs::read_to_string(&data_path)
                    .context("Failed to read easy_sql.ron file")?;

                let mut data: CompilationData =
                    ron::de::from_str(&data).context("Failed to parse easy_sql.ron file")?;

                if default_drivers_update && data.default_drivers != default_drivers {
                    data.default_drivers = default_drivers;
                    data.save()?;
                }

                data
            }
        };

        Ok(data)
    }

    /// Parses the current default-driver strings into Rust paths.
    pub fn default_driver_paths(&self) -> anyhow::Result<Vec<syn::Path>> {
        self.default_drivers
            .iter()
            .map(|driver_str| {
                syn::parse_str(driver_str).with_context(|| {
                    format!(
                        "Failed to parse default driver `{}`. Expected a valid Rust path. (easy_sql.ron is corrupted)",
                        driver_str
                    )
                })
            })
            .collect()
    }

    pub fn load_in_macro() -> anyhow::Result<CompilationData> {
        let data_path = Self::data_location()?;

        let data: CompilationData = {
            {
                if !data_path.exists() {
                    return Ok(CompilationData {
                        tables: HashMap::new(),
                        #[cfg(feature = "check_duplicate_table_names")]
                        used_table_names: HashMap::new(),
                        default_drivers: Vec::new(),
                    });
                }

                let data = std::fs::read_to_string(&data_path)
                    .context("Failed to read easy_sql.ron file")?;

                ron::de::from_str(&data).context("Failed to parse easy_sql.ron file")?
            }
        };

        Ok(data)
    }

    #[cfg(feature = "build")]
    pub fn save(&self) -> anyhow::Result<()> {
        let data_path = Self::data_location()?;

        let data =
            ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::new().struct_names(true))?;

        let result = std::fs::write(&data_path, &data);

        if let Err(e) = &result
            && let std::io::ErrorKind::ReadOnlyFilesystem = e.kind()
        {
            return Ok(());
        }

        result.context("Failed to write easy_sql.ron file")?;

        Ok(())
    }

    pub fn generate_unique_id(&self) -> String {
        let mut generated = uuid::Uuid::new_v4().to_string();
        let mut exists = true;

        while exists {
            exists = false;
            for unique_id in self.tables.keys() {
                if unique_id == &generated {
                    exists = true;
                    generated = uuid::Uuid::new_v4().to_string();
                    break;
                }
            }
        }
        generated
    }

    pub fn is_duplicate_table_name(
        &self,
        current_unique_id: &str,
        table_name: &str,
    ) -> anyhow::Result<bool> {
        if table_name == "easy_sql_tables" {
            return Ok(true);
        }
        for (unique_id, table_data) in self.tables.iter() {
            if unique_id == current_unique_id {
                continue;
            }
            let latest_version_data =
                match table_data.saved_versions.get(&table_data.latest_version) {
                    Some(o) => o,
                    None => anyhow::bail!(
                        "Table data not found for latest version: {} | unique id: {:?}",
                        table_data.latest_version,
                        unique_id
                    ),
                };

            if latest_version_data.table_name == table_name {
                return Ok(true);
            }
        }

        Ok(false)
    }

    #[cfg(feature = "migrations")]
    pub fn generate_migrations(
        &self,
        current_unique_id: &str,
        latest_version: &TableDataVersion,
        latest_version_number: i64,
        sql_crate: &TokenStream,
        item_name: &TokenStream,
    ) -> anyhow::Result<TokenStream> {
        let macro_support = quote! { #sql_crate::macro_support };

        let table_data = self
            .tables
            .get(current_unique_id)
            .context("Table not found in Sql Compilation Data (easy_sql.ron)")?;

        let mut result = TokensBuilder::default();

        for (version_number, version_data) in table_data.saved_versions.iter() {
            let mut changes_needed = Vec::new();
            let mut rename_table = None;

            if version_number == &latest_version_number {
                continue;
            }
            //Primary Key Check (Must be equal)
            if version_data.primary_keys != latest_version.primary_keys {
                anyhow::bail!(
                    "Primary key change is not supported (yet) -> Latest Version: {:?} ||| Version {}: {:?}",
                    latest_version.primary_keys,
                    version_number,
                    version_data.primary_keys
                );
            }
            // Foreign key diff. Only ADDING a foreign key is generated (a table rebuild on SQLite / ADD CONSTRAINT
            // on Postgres): emit one for every foreign key present in the latest version but absent from this older
            // saved version. Every other difference is intentionally ignored — mirroring how a column removal is a
            // no-op here — which is also what keeps this correct while expanding an OLDER version's struct (where
            // the `latest_version` passed in legitimately has FEWER foreign keys than a newer saved version). The
            // snapshot key encodes the referenced struct plus an optional `,cascade` (e.g. "NoteFolder" or
            // "NoteFolder,cascade"), so the on-delete action is parsed from it. Applied AFTER the column changes
            // below (so a rebuild sees any newly-added columns) — see where `fk_additions` is drained.
            let mut fk_additions = Vec::new();
            for (fk_key, latest_columns) in latest_version.foreign_keys.iter() {
                if version_data.foreign_keys.contains_key(fk_key) {
                    continue; // already present in this older version — unchanged, nothing to add
                }
                let cascade = fk_key.split(',').any(|part| part.trim() == "cascade");
                let struct_name = fk_key.split(',').next().unwrap_or(fk_key.as_str()).trim();
                let fk_path: syn::Path = syn::parse_str(struct_name).with_context(|| {
                    format!("Foreign key target `{struct_name}` is not a valid type path")
                })?;
                let local_columns: Vec<&str> = latest_columns.iter().map(String::as_str).collect();
                fk_additions.push(quote! {
                    #sql_crate::driver::AlterTableSingle::AddForeignKey {
                        columns: vec![#(#local_columns),*],
                        referenced_table: <#fk_path as #sql_crate::Table<_EasySqlMigrationDriver>>::table_name(),
                        referenced_columns: <#fk_path as #sql_crate::Table<_EasySqlMigrationDriver>>::primary_keys(),
                        cascade: #cascade,
                    }
                });
            }
            //Auto increment check (Must be equal)
            if version_data.auto_increment != latest_version.auto_increment {
                anyhow::bail!(
                    "Auto increment change is not supported (yet) -> Latest Version: {:?} ||| Version {}: {:?}",
                    latest_version.auto_increment,
                    version_number,
                    version_data.auto_increment
                );
            }

            // Table name change support
            if version_data.table_name != latest_version.table_name {
                let new_name = latest_version.table_name.as_str();

                rename_table = Some(quote! {
                    #sql_crate::driver::AlterTableSingle::RenameTable{
                        new_table_name: #new_name,
                    }
                });
            }
            // Check for old column change
            for (old_field, new_field) in
                version_data.fields.iter().zip(latest_version.fields.iter())
            {
                //We can only rename old columns
                if old_field.name != new_field.name {
                    let old_name = old_field.name.as_str();
                    let new_name = new_field.name.as_str();

                    changes_needed.push(quote! {
                        #sql_crate::driver::AlterTableSingle::RenameColumn{
                            old_column_name: #old_name,
                            new_column_name: #new_name,
                        }
                    });
                }
                //Everything else on old column is not supported
                // Compare persisted storage compatibility so `#[sql(bytes)]` wrapper changes do not trip the unsupported type-change guard.
                if !old_field.is_migration_storage_compatible_with(new_field) {
                    anyhow::bail!(
                        "Field type change is not supported (yet) (only rename) -> Latest Version: {:?} ||| Version {}: {:?}",
                        latest_version.fields,
                        version_number,
                        version_data.fields
                    );
                }
                if old_field.is_unique != new_field.is_unique {
                    anyhow::bail!(
                        "Field unique change is not supported (yet) (only rename) -> Latest Version: {:?} ||| Version {}: {:?}",
                        latest_version.fields,
                        version_number,
                        version_data.fields
                    );
                }
                if old_field.default != new_field.default {
                    anyhow::bail!(
                        "Field default value change is not supported (yet) (only rename) -> Latest Version: {:?} ||| Version {}: {:?}",
                        latest_version.fields,
                        version_number,
                        version_data.fields
                    );
                }
            }

            //New Columns Check
            for new_field in latest_version.fields.iter().skip(version_data.fields.len()) {
                //New columns need default value
                if new_field.default.is_none() && !new_field.field_type.starts_with("Option<") {
                    anyhow::bail!(
                        "New (not null) column without default value is not supported -> Latest Version: {:?} ||| Version {}: {:?}",
                        latest_version.fields,
                        version_number,
                        version_data.fields
                    );
                }

                let field_name = new_field.name.as_str();
                let field_ident = syn::Ident::new(field_name, Span::call_site());
                let data_type: syn::Type = syn::parse_str(new_field.field_type.as_str())?;
                let is_not_null = !new_field.field_type.starts_with("Option<");
                let is_unique = new_field.is_unique;

                let default_value = if let Some(default_value) = new_field.default.as_deref() {
                    let default_expr: syn::Expr = syn::parse_str(default_value)?;

                    //For compatibility sake
                    let default_value = default_expr;
                    let default_context = format!(
                        "Converting default value for field `{}` in struct `{}` (table `{}`)",
                        field_name, item_name, latest_version.table_name
                    );

                    quote! {
                        {
                            //Check if default value has valid type for the current column
                            let _= ||{
                                let mut table_instance = #macro_support::never_any::<#item_name>();
                                table_instance.#field_ident = #default_value;
                            };

                            Some(#macro_support::Context::context(
                                #sql_crate::ToDefault::to_default_failable(#default_value),
                                #default_context,
                            )?)
                        }
                    }
                } else {
                    quote! {
                        None
                    }
                };

                //Create new field
                changes_needed.push(quote! {
                    #sql_crate::driver::AlterTableSingle::AddColumn{
                        column: #sql_crate::driver::TableField {
                            name: #field_name,
                            data_type: {
                                #macro_support::TypeInfo::name(
                                    &<#data_type as #macro_support::Type<#macro_support::InternalDriver<_EasySqlMigrationDriver>>>::type_info(),
                                )
                                .to_owned()
                            },
                            is_unique: #is_unique,
                            is_not_null: #is_not_null,
                            default: #default_value,
                            is_auto_increment: false,
                        }
                    }
                });
            }

            // Apply foreign-key additions after the column changes (a SQLite rebuild reads the table's current
            // DDL, so any AddColumn above must already be applied) but before a rename.
            changes_needed.append(&mut fk_additions);

            if let Some(rename_table) = rename_table {
                changes_needed.push(rename_table);
            }

            let version_number = *version_number;
            let table_name = version_data.table_name.as_str();
            let has_schema_changes = !changes_needed.is_empty();

            let apply_alter = if has_schema_changes {
                quote! {
                    #sql_crate::EasyExecutor::query_setup(conn, #sql_crate::driver::AlterTable{
                        table_name: #table_name,
                        alters: vec![#(#changes_needed),*],
                    })
                    .await
                    .with_context(#macro_support::context!(
                        "setup failed: operation=apply_migration_alter, table={}, unique_id={}, from_version={}, to_version={}, driver={}",
                        #table_name,
                        #current_unique_id,
                        #version_number,
                        #latest_version_number,
                        stringify!(_EasySqlMigrationDriver)
                    ))?;
                }
            } else {
                quote! {}
            };

            result.add(quote! {
                if current_version_number == #version_number{
                    use #macro_support::Context;
                    #apply_alter

                    {
                        let __easy_sql_update_version_result: #macro_support::Result<()> = async {
                            #sql_crate::EasySqlTables_update_version!(_EasySqlMigrationDriver, *conn, #current_unique_id, #latest_version_number);
                            Ok(())
                        }
                        .await;

                        __easy_sql_update_version_result.with_context(#macro_support::context!(
                            "setup failed: operation=update_version_after_migration, table={}, unique_id={}, from_version={}, to_version={}, driver={}",
                            #table_name,
                            #current_unique_id,
                            #version_number,
                            #latest_version_number,
                            stringify!(_EasySqlMigrationDriver)
                        ))?;
                    }
                }
            });
        }

        Ok(result.finalize())
    }
}
