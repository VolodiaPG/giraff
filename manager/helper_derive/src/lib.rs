#[allow(refining_impl_trait)]
use helper::monitoring::InfluxName;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::token::Pub;
use syn::{parse, parse_macro_input, FieldsNamed, ItemStruct};

#[proc_macro_attribute]
pub fn influx_observation(
    args: TokenStream,
    input: TokenStream,
) -> TokenStream {
    let mut item_struct = parse_macro_input!(input as ItemStruct);
    let mut item_struct_to_db = item_struct.clone();
    let _ = parse_macro_input!(args as parse::Nothing);

    let syn::Fields::Named(ref mut fields) = item_struct.fields else {
        unimplemented!("Only works for structs");
    };
    fields.named.push(
        syn::Field::parse_named
            .parse2(quote! { pub timestamp: chrono::DateTime<chrono::Utc> })
            .unwrap(),
    );
    let FieldsNamed { ref named, .. } = fields.clone();
    let original_struct = named.iter().map(|f| {
        let name = &f.ident;
        let typ = &f.ty;

        quote! { pub #name: #typ }
    });

    let syn::Fields::Named(ref mut fields) = item_struct_to_db.fields else {
        unimplemented!("Only works for structs");
    };
    fields.named.push(
        syn::Field::parse_named
            .parse2(quote! { #[influxdb(timestamp)] pub timestamp: i64 })
            .unwrap(),
    );
    fields.named.push(
        syn::Field::parse_named
            .parse2(quote! { #[influxdb(tag)] pub instance: String })
            .unwrap(),
    );
    let FieldsNamed { ref named, .. } = fields.clone();
    let builder_fields = named
        .iter()
        .filter(|field| {
            if let Some(name) = &field.ident {
                return *name != "timestamp" && *name != "instance";
            }
            true
        })
        .map(|f| {
            let name = &f.ident;

            quote! { #name }
        });

    let name = &item_struct.ident;
    let influx_name = InfluxName::try_new(name.to_string())
        .expect(
            "Provided name is not conform to the validation and sanitization \
             schemas",
        )
        .into_inner();
    item_struct_to_db.ident = syn::Ident::new(
        &format!("{}Exported", name.to_string()),
        item_struct_to_db.ident.span(),
    );
    item_struct_to_db.vis =
        syn::Visibility::Public(Pub { span: item_struct_to_db.ident.span() });
    let exported_struct_name = &item_struct_to_db.ident;

    quote! {
        pub struct #name { // set the struct to public
            #(#original_struct,)*
        }
        #[derive(influxdb2_derive::WriteDataPoint)]
        #[measurement = #influx_name]
        #item_struct_to_db
        impl helper::monitoring::InfluxData for #name {
            fn export(self, instance: String) -> impl influxdb2::models::WriteDataPoint + Sync + Send + 'static {
                #exported_struct_name {
                    #(#builder_fields: self.#builder_fields,)*
                    instance,
                    timestamp: helper::monitoring::convert_timestamp(self.timestamp),
                }
            }
        }
    }
    .into()
}
