use uom::si::f64::{Information, Ratio};
use uom::{fmt::DisplayStyle::Abbreviation, si::information};

pub use function_definition::{FunctionDefinition, Limits};

pub use self::function_list_entry::FunctionListEntry;

pub struct InformationHelper;

impl serde_with::SerializeAs<Information> for InformationHelper {
    fn serialize_as<S>(value: &Information, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Do not forget to remove last letter of the sentence (the unit)
        serializer.serialize_str(&format!(
            "{}M",
            &format!(
                "{:?}",
                value.into_format_args(information::megabyte, Abbreviation)
            )
            .split_whitespace()
            .collect::<Vec<&str>>()[0]
        ))
    }
}

pub struct RatioHelper;

impl serde_with::SerializeAs<Ratio> for RatioHelper {
    fn serialize_as<S>(value: &Ratio, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Do not forget to remove last letter of the sentence (the unit)
        serializer.serialize_str(
            (&format!(
                "{:?}",
                value.into_format_args(crate::helper::uom::cpu_ratio::cpu, Abbreviation)
            )
            .split_whitespace()
            .collect::<Vec<&str>>()[0])
                .as_ref(),
        )
    }
}

mod function_definition;

mod function_list_entry;
