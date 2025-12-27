#[macro_export(local_inner_macros)]
macro_rules! sqlite_math_required {
    () => {
        #[cfg(feature = "sqlite")]
        {
            if std::option_env!("LIBSQLITE3_FLAGS")
                .map(|flags| flags.contains("-DSQLITE_ENABLE_MATH_FUNCTIONS"))
                .unwrap_or(false)
                == false
            {
                return Ok(());
            }
        }
    };
}
