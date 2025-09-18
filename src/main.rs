#![feature(iter_collect_into)]

use asf::{
    app::{subscription, update, view, Flags, State},
    skill::Skill,
};
use iced::{window, Task, Theme};
use itertools::Itertools;
use serde::de::DeserializeOwned;
use std::error::Error;
fn read_to_vec<T>(path: &str, skip: usize) -> Vec<T>
where
    T: DeserializeOwned,
{
    csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)
        .map(|mut x| x.deserialize().skip(skip).flatten().collect())
        .unwrap_or_default()
}

fn main() -> Result<(), Box<dyn Error>> {
    let skills: Vec<Skill> = csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .from_path("./skills.txt")
        .unwrap()
        .deserialize()
        .skip(1)
        .flatten()
        .collect();
    let body = read_to_vec("./body.txt", 1);
    let head = read_to_vec("./head.txt", 1);
    let arms = read_to_vec("./arms.txt", 1);
    let waist = read_to_vec("./waist.txt", 1);
    let legs = read_to_vec("./legs.txt", 1);
    let relic_body = read_to_vec("./relic_body.txt", 0);
    let relic_head = read_to_vec("./relic_head.txt", 0);
    let relic_arms = read_to_vec("./relic_arms.txt", 0);
    let relic_waist = read_to_vec("./relic_waist.txt", 0);
    let relic_legs = read_to_vec("./relic_legs.txt", 0);
    let charms = read_to_vec("./mycharms.txt", 1);
    let decorations = read_to_vec("./decorations.txt", 1);
    let components = csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .from_path("./components.txt")
        .unwrap()
        .deserialize()
        .flatten()
        .collect_vec();
    let data = Flags {
        body,
        head,
        arms,
        waist,
        legs,
        relic_body,
        relic_head,
        relic_arms,
        relic_legs,
        relic_waist,
        charms,
        decorations,
        skills,
        components,
    };
    let window = window::Settings {
        exit_on_close_request: false,
        ..window::Settings::default()
    };
    let state = State::new(data);
    iced::application("Armor Set Finder", update, view)
        .theme(|_| Theme::SolarizedLight)
        .subscription(subscription)
        .window(window)
        .run_with(|| (state, Task::none()))
        .unwrap();
    Ok(())
}
