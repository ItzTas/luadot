use std::ptr;

use mlua::{LightUserData, Value};

pub fn value() -> Value {
    Value::LightUserData(LightUserData(ptr::null_mut()))
}

pub fn is_null(value: &Value) -> bool {
    matches!(value, Value::LightUserData(data) if data.0.is_null())
}
