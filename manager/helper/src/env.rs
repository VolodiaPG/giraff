#[macro_export]
macro_rules! env_var {
    ($name:ident) => {
        const $name: &'static str = stringify!($name);
    };
}
#[macro_export]
macro_rules! env_load {
    ($type:ident, $name:ident) => {
        $type::new(
            std::env::var($name)
                .with_context(|| format!("Missing {} env var", $name))?,
        )
        .with_context(|| format!("{} was not formatted right", $name))?
    };
    ($type:ident, $name:ident, $type_raw:ident) => {
        $type::new(
            std::env::var($name)
                .with_context(|| format!("Missing {} env var", $name))?
                .parse::<$type_raw>()
                .with_context(|| {
                    format!(
                        "{} env var cannot be parsed in the correct type",
                        $name
                    )
                })?,
        )
        .with_context(|| format!("{} was not formatted right", $name))?
    };
}
