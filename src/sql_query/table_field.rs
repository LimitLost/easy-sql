use anyhow::Context;
use easy_macros::{helpers::context, macros::always_context};
use sql_compilation_data::SqlType;

use crate::{Driver, DriverValue};

#[derive(Debug)]
pub struct TableField<'a, D: Driver> {
    pub name: &'static str,
    pub data_type: SqlType,
    pub is_unique: bool,
    pub is_not_null: bool,
    pub default: Option<&'a D::Value<'a>>,
}

#[always_context]
impl<'a, D: Driver> TableField<'a, D> {
    pub fn definition(self) -> anyhow::Result<String> {
        let TableField {
            name,
            data_type,
            is_unique,
            is_not_null,
            default,
        } = self;

        let date_type_sqlite = data_type.sqlite();

        let unique = if is_unique { "UNIQUE" } else { "" };
        let not_null = if is_not_null { "NOT NULL" } else { "" };
        let default = if let Some(default) = default {
            format!(
                "DEFAULT {}",
                default
                    .to_default()
                    .with_context(context!("field name: {}", name))?
            )
        } else {
            String::new()
        };

        Ok(format!(
            "{} {} {} {} {},",
            name, date_type_sqlite, unique, not_null, default
        ))
    }
}

#[derive(Debug)]
pub struct ForeignKey {
    pub table_name: &'static str,
    pub referenced_field: &'static str,
}
