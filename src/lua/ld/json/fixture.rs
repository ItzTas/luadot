use super::super::fixture;
use super::constants::NAMESPACE;
use super::table::table;

pub fn eval(source: &str) -> mlua::Result<String> {
    fixture::eval(NAMESPACE, table, source)
}
