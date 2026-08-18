use mlua::{Lua, Table};

use super::card::{Card, cards};
use super::constants::{DRIVER, NAME, VENDOR};
use super::model::models;

type Field = (&'static str, fn(&Card) -> &str);

pub fn table(lua: &Lua) -> mlua::Result<Table> {
    build(lua, &cards(models()))
}

fn build(lua: &Lua, cards: &[Card]) -> mlua::Result<Table> {
    let fields: [Field; 3] = [
        (VENDOR, |card| &card.vendor),
        (NAME, |card| &card.name),
        (DRIVER, |card| &card.driver),
    ];

    let gpu = lua.create_table()?;
    for card in cards {
        gpu.raw_push(entry(lua, card, &fields)?)?;
    }

    for (key, read) in fields {
        gpu.set(key, cards.first().map(read).unwrap_or_default())?;
    }

    Ok(gpu)
}

fn entry(lua: &Lua, card: &Card, fields: &[Field]) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (key, read) in fields {
        table.set(*key, read(card))?;
    }

    Ok(table)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua::runtime::runtime;

    fn card(vendor: &str, name: &str, driver: &str) -> Card {
        Card {
            vendor: vendor.to_string(),
            name: name.to_string(),
            driver: driver.to_string(),
        }
    }

    #[test]
    fn the_first_card_answers_the_namespace_itself() {
        let lua = runtime().unwrap();
        let found = [
            card("intel", "Raptor Lake-P [Iris Xe]", "i915"),
            card("nvidia", "AD107M [GeForce RTX 4060]", "nvidia"),
        ];

        let gpu = build(&lua, &found).unwrap();

        assert_eq!(gpu.get::<String>(VENDOR).unwrap(), "intel");
        assert_eq!(gpu.get::<String>(NAME).unwrap(), "Raptor Lake-P [Iris Xe]");
        assert_eq!(gpu.get::<String>(DRIVER).unwrap(), "i915");
    }

    #[test]
    fn every_card_is_an_element_of_the_namespace() {
        let lua = runtime().unwrap();
        let found = [
            card("intel", "Raptor Lake-P [Iris Xe]", "i915"),
            card("nvidia", "AD107M [GeForce RTX 4060]", "nvidia"),
        ];

        let gpu = build(&lua, &found).unwrap();

        assert_eq!(gpu.raw_len(), 2);
        assert_eq!(
            gpu.get::<Table>(2).unwrap().get::<String>(VENDOR).unwrap(),
            "nvidia"
        );
    }

    #[test]
    fn a_machine_without_a_card_answers_empty_strings() {
        let lua = runtime().unwrap();

        let gpu = build(&lua, &[]).unwrap();

        assert_eq!(gpu.raw_len(), 0);
        for key in [VENDOR, NAME, DRIVER] {
            assert_eq!(gpu.get::<String>(key).unwrap(), "");
        }
    }
}
