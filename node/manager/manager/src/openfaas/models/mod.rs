use uom::si::f64::Information;
use uom::{fmt::DisplayStyle::Description, si::information};

pub struct Helper;

impl serde_with::SerializeAs<Information> for Helper {
    fn serialize_as<S>(value: &Information, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Do not forget to remove last letter of the sentence (the unit)
        serializer.serialize_str(&format!(
            "{}M",
            &format!(
                "{:?}",
                value.into_format_args(information::megabyte, Description)
            )
            .split_whitespace()
            .collect::<Vec<&str>>()[0]
        ))
    }
}

mod function_definition;
pub use function_definition::{FunctionDefinition, Limits};
