use serde::{Deserialize, Serialize};

use crate::{component::Component, requirements::Requirement, skillpoint::SkillPoint};

#[derive(Serialize, Debug, PartialEq, Eq, Hash, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Armor {
    pub name: String,
    pub sex: i32,
    pub weapon_type: i32,
    pub rarity: i32,
    pub slots: i32,
    pub hunter_rank: i32,
    pub village_rank: i32,
    pub defence_min: i32,
    pub defence_max: i32,
    pub defence_fire: i32,
    pub defence_water: i32,
    pub defence_thunder: i32,
    pub defence_ice: i32,
    pub defence_dragon: i32,
    pub ability_1_name: String,
    pub ability_1_points: Option<i32>,
    pub ability_2_name: String,
    pub ability_2_points: Option<i32>,
    pub ability_3_name: String,
    pub ability_3_points: Option<i32>,
    pub ability_4_name: String,
    pub ability_4_points: Option<i32>,
    pub ability_5_name: String,
    pub ability_5_points: Option<i32>,
    pub material_1_name: String,
    pub material_1_count: Option<i32>,
    pub material_2_name: String,
    pub material_2_count: Option<i32>,
    pub material_3_name: String,
    pub material_3_count: Option<i32>,
    pub material_4_name: String,
    pub material_4_count: Option<i32>,
}

impl Armor {
    pub fn is_valid(
        &self,
        hr: i32,
        village: i32,
        min_rarity: i32,
        sex: i32,
        japanese: bool,
        weapon_type: i32,
        components: &[Component],
        requirements: &[Requirement],
    ) -> bool {
        if (self.hunter_rank > hr && self.village_rank > village)
            || (self.sex != 0 && self.sex != sex)
            || (self.weapon_type != 0 && self.weapon_type != weapon_type)
            || (self.japanese(components) && !japanese)
            || self.rarity < min_rarity
        {
            false
        } else {
            if requirements.is_empty() {
                return true;
            }
            for requirement in requirements {
                if self.points(&requirement.name) > 0 {
                    return true;
                }
            }
            false
        }
    }
    pub fn japanese(&self, components: &[Component]) -> bool {
        [
            &self.material_1_name,
            &self.material_2_name,
            &self.material_3_name,
            &self.material_4_name,
        ]
        .into_iter()
        .any(|name| {
            components
                .iter()
                .find(|&c| &c.name == name)
                .map(|c| c.japanese())
                .unwrap_or(false)
        })
    }
}

impl SkillPoint for Armor {
    fn points(&self, ability: &str) -> i32 {
        if ability == self.ability_5_name {
            self.ability_5_points
        } else if ability == self.ability_4_name {
            self.ability_4_points
        } else if ability == self.ability_3_name {
            self.ability_3_points
        } else if ability == self.ability_2_name {
            self.ability_2_points
        } else if ability == self.ability_1_name {
            self.ability_1_points
        } else {
            Some(0)
        }
        .unwrap_or(0)
    }
    fn slots(&self) -> i32 {
        self.slots
    }

    fn translate_skills(&mut self, translation: &std::collections::HashMap<String, String>) {
        if let Some(t) = translation.get(&self.ability_1_name) {
            self.ability_1_name.clone_from(t);
        }
        if let Some(t) = translation.get(&self.ability_2_name) {
            self.ability_2_name.clone_from(t);
        }
        if let Some(t) = translation.get(&self.ability_3_name) {
            self.ability_3_name.clone_from(t);
        }
        if let Some(t) = translation.get(&self.ability_4_name) {
            self.ability_4_name.clone_from(t);
        }
        if let Some(t) = translation.get(&self.ability_5_name) {
            self.ability_5_name.clone_from(t);
        }
    }

    fn max_defence(&self) -> u32 {
        self.defence_max as u32
    }
    fn defence(&self) -> u32 {
        self.defence_min as u32
    }
}
