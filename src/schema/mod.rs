use schemars::{
    JsonSchema, SchemaGenerator,
    schema::{InstanceType, Schema, SchemaObject},
};
use serde::{Deserialize, Serialize};

pub mod user;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Pagination {
    pub max_items: usize,
    pub cursor: Cursor,
}

#[derive(Debug, Clone)]
pub struct Cursor {
    pub key_id: u32,
    pub offset: u32,
    pub limit: u32,
}

// We need to make sure the cursor is opaque so that clients don't
// rely on the implementation details.
impl Serialize for Cursor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}

impl<'de> Deserialize<'de> for Cursor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
}

impl JsonSchema for Cursor {
    fn schema_name() -> String {
        "Cursor".into()
    }

    fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
        let schema = SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            ..Default::default()
        };

        Schema::Object(schema)
    }
}
