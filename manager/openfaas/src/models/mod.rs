pub use self::function_list_entry::FunctionListEntry;
pub use function_definition::{FunctionDefinition, Limits, Requests};
use uom::num_traits::ToPrimitive;
use uom::si::information;
use uom::si::rational64::{Information, Ratio};

pub struct InformationHelper;
pub struct RatioHelper;

impl serde_with::SerializeAs<Information> for InformationHelper {
    fn serialize_as<S>(
        value: &Information,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Do not forget to remove last letter of the sentence (the unit)
        serializer.serialize_str(&format!(
            "{}M",
            value.get::<information::megabyte>()
        ))
    }
}

impl serde_with::SerializeAs<Ratio> for RatioHelper {
    fn serialize_as<S>(value: &Ratio, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Do not forget to remove last letter of the sentence (the unit)
        serializer.serialize_str(&format!(
            "{:?}m",
            value
                .get::<helper::uom_helper::cpu_ratio::millicpu>()
                .to_f64()
                .unwrap()
        ))
    }
}

pub mod delete_function_request;
pub mod function_definition;
pub mod function_list_entry;
