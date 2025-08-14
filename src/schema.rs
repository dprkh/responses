use schemars::{
    JsonSchema, Schema,
    generate::SchemaSettings,
    transform::{RecursiveTransform, ReplaceConstValue},
};

pub fn from<T: ?Sized + JsonSchema>() -> Schema {
    SchemaSettings::default()
        //
        .for_serialize()
        //
        .with(|s| s.meta_schema = None)
        //
        .with_transform(ReplaceConstValue::default())
        //
        .with_transform(RecursiveTransform(|schema: &mut Schema| {
            if schema.get("properties").is_some() {
                schema.insert("additionalProperties".to_owned(), false.into());
            }

            if let Some(one_of) = schema.remove("oneOf") {
                schema.insert("anyOf".to_owned(), one_of);
            }
        }))
        //
        .into_generator()
        //
        .into_root_schema_for::<T>()
}
