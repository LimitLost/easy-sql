/// This module provides built-in SQL function support for the query! macro.
///
/// These functions are automatically available and can be used directly in SQL expressions.
/// Some functions (like COUNT) can accept the special `*` wildcard argument.
///
use lazy_static::lazy_static;
/// Argument count support is handled by function specific traits
pub struct BuiltinFunctionData {
    pub name: &'static str,
    pub accepts_star: bool,
    pub maybe_value: bool,
}

impl BuiltinFunctionData {
    pub fn count() -> Self {
        BuiltinFunctionData {
            name: "COUNT",
            accepts_star: true,
            maybe_value: false,
        }
    }

    pub fn new(name: &'static str) -> Self {
        BuiltinFunctionData {
            name,
            accepts_star: false,
            maybe_value: false,
        }
    }

    pub fn new_maybe_value(name: &'static str) -> Self {
        BuiltinFunctionData {
            name,
            accepts_star: false,
            maybe_value: true,
        }
    }
}
lazy_static! {
    static ref BUILTIN_FUNCTIONS: Vec<BuiltinFunctionData> = vec![
        // Aggregate functions
        BuiltinFunctionData::count(), // COUNT(*) or COUNT(column) or COUNT()
        BuiltinFunctionData::new("SUM"),
        BuiltinFunctionData::new("AVG"),
        BuiltinFunctionData::new("MIN"),
        BuiltinFunctionData::new("MAX"),
        // String functions
        BuiltinFunctionData::new("CONCAT"), // 1 or more arguments
        BuiltinFunctionData::new("UPPER"),
        BuiltinFunctionData::new("LOWER"),
        BuiltinFunctionData::new("LENGTH"),
        BuiltinFunctionData::new("TRIM"),
        BuiltinFunctionData::new("SUBSTRING"), // SUBSTRING(str, start) or SUBSTRING(str, start, length)
        BuiltinFunctionData::new("SUBSTR"),    // Alias for SUBSTRING
        // Conditional functions
        BuiltinFunctionData::new("COALESCE"), // 1 or more arguments
        BuiltinFunctionData::new("NULLIF"),
        BuiltinFunctionData::new("IFNULL"), // SQLite
        // Date/Time functions
        BuiltinFunctionData::new("NOW"),
        BuiltinFunctionData::new("DATE"),
        BuiltinFunctionData::new("TIME"),
        BuiltinFunctionData::new("DATETIME"),
        BuiltinFunctionData::new_maybe_value("CURRENT_TIMESTAMP"),
        BuiltinFunctionData::new_maybe_value("CURRENT_DATE"),
        BuiltinFunctionData::new_maybe_value("CURRENT_TIME"),
        // Math functions
        BuiltinFunctionData::new("ABS"),
        BuiltinFunctionData::new("ROUND"), // ROUND(num) or ROUND(num, decimals)
        BuiltinFunctionData::new("CEIL"),
        BuiltinFunctionData::new("CEILING"),
        BuiltinFunctionData::new("FLOOR"),
        BuiltinFunctionData::new("POWER"),
        BuiltinFunctionData::new("POW"),
        BuiltinFunctionData::new("SQRT"),
        BuiltinFunctionData::new("MOD"),
        // Type conversion
        BuiltinFunctionData::new("CAST"), // Note: CAST has special syntax CAST(expr AS type)
        // Other common functions
        BuiltinFunctionData::new("DISTINCT"),

    ];
}

pub fn get_builtin_fn(name: &str) -> Option<&BuiltinFunctionData> {
    let upper_name = name.to_uppercase();
    BUILTIN_FUNCTIONS
        .iter()
        .find(|func| func.name == upper_name)
}
