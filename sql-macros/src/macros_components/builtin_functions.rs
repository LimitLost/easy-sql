/// This module provides built-in SQL function support for the query! macro.
///
/// These functions are automatically available and can be used directly in SQL expressions.
/// Some functions (like COUNT) can accept the special `*` wildcard argument.
///
use lazy_static::lazy_static;

pub struct BuiltinFunctionData {
    pub name: &'static str,
    pub min_args: usize,
    pub max_args: Option<usize>, // None means any number of args
    pub accepts_star: bool,
}

impl BuiltinFunctionData {
    pub fn count() -> Self {
        BuiltinFunctionData {
            name: "COUNT",
            min_args: 0,
            max_args: Some(1),
            accepts_star: true,
        }
    }

    pub fn args(name: &'static str, min_args: usize, max_args: Option<usize>) -> Self {
        BuiltinFunctionData {
            name,
            min_args,
            max_args,
            accepts_star: false,
        }
    }

    pub fn args1(name: &'static str) -> Self {
        BuiltinFunctionData {
            name,
            min_args: 1,
            max_args: Some(1),
            accepts_star: false,
        }
    }

    pub fn args2(name: &'static str) -> Self {
        BuiltinFunctionData {
            name,
            min_args: 2,
            max_args: Some(2),
            accepts_star: false,
        }
    }
}
lazy_static! {
    static ref BUILTIN_FUNCTIONS: Vec<BuiltinFunctionData> = vec![
        // Aggregate functions
        BuiltinFunctionData::count(), // COUNT(*) or COUNT(column) or COUNT()
        BuiltinFunctionData::args1("SUM"),
        BuiltinFunctionData::args1("AVG"),
        BuiltinFunctionData::args1("MIN"),
        BuiltinFunctionData::args1("MAX"),
        // String functions
        BuiltinFunctionData::args("CONCAT", 1, None), // 1 or more arguments
        BuiltinFunctionData::args1("UPPER"),
        BuiltinFunctionData::args1("LOWER"),
        BuiltinFunctionData::args1("LENGTH"),
        BuiltinFunctionData::args1("TRIM"),
        BuiltinFunctionData::args("SUBSTRING", 2, Some(3)), // SUBSTRING(str, start) or SUBSTRING(str, start, length)
        BuiltinFunctionData::args("SUBSTR", 2, Some(3)),    // Alias for SUBSTRING
        // Conditional functions
        BuiltinFunctionData::args("COALESCE", 1, None), // 1 or more arguments
        BuiltinFunctionData::args2("NULLIF"),
        BuiltinFunctionData::args2("IFNULL"), // SQLite
        // Date/Time functions
        BuiltinFunctionData::args("NOW", 0, Some(0)),
        BuiltinFunctionData::args1("DATE"),
        BuiltinFunctionData::args1("TIME"),
        BuiltinFunctionData::args1("DATETIME"),
        BuiltinFunctionData::args("CURRENT_TIMESTAMP", 0, Some(0)),
        BuiltinFunctionData::args("CURRENT_DATE", 0, Some(0)),
        BuiltinFunctionData::args("CURRENT_TIME", 0, Some(0)),
        // Math functions
        BuiltinFunctionData::args1("ABS"),
        BuiltinFunctionData::args("ROUND", 1, Some(2)), // ROUND(num) or ROUND(num, decimals)
        BuiltinFunctionData::args1("CEIL"),
        BuiltinFunctionData::args1("CEILING"),
        BuiltinFunctionData::args1("FLOOR"),
        BuiltinFunctionData::args2("POWER"),
        BuiltinFunctionData::args2("POW"),
        BuiltinFunctionData::args1("SQRT"),
        BuiltinFunctionData::args2("MOD"),
        // Type conversion
        BuiltinFunctionData::args1("CAST"), // Note: CAST has special syntax CAST(expr AS type)
        // Other common functions
        BuiltinFunctionData::args1("DISTINCT"),

    ];
}

pub fn get_builtin_fn(name: &str) -> Option<&BuiltinFunctionData> {
    let upper_name = name.to_uppercase();
    BUILTIN_FUNCTIONS
        .iter()
        .find(|func| func.name == upper_name)
}
