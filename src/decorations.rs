use serde::Deserialize;

use crate::{component::Component, requirements::Requirement, skillpoint::SkillPoint};

#[derive(Default, PartialEq, Eq, Hash, Clone, Deserialize, Debug)]
#[serde(default)]
pub struct Decoration {
    pub name: String,
    rarity: i32,
    slots: i32,
    pub hunter_rank: i32,
    pub village_rank: i32,
    skill_1_name: String,
    skill_1_points: i32,
    skill_2_name: String,
    skill_2_points: Option<i32>,
    material_a1_name: String,
    material_a1_count: Option<i32>,
    material_a2_name: String,
    material_a2_count: Option<i32>,
    material_a3_name: String,
    material_a3_count: Option<i32>,
    material_a4_name: String,
    material_a4_count: Option<i32>,
    material_b1_name: String,
    material_b1_count: Option<i32>,
    material_b2_name: String,
    material_b2_count: Option<i32>,
    material_b3_name: String,
    material_b3_count: Option<i32>,
    material_b4_name: String,
    material_b4_count: Option<i32>,
}
impl Decoration {
    pub fn is_valid(
        &self,
        hr: i32,
        village: i32,
        _japanese: bool,
        _components: &[Component],
        requirements: &[Requirement],
    ) -> bool {
        if self.hunter_rank > hr && self.village_rank > village {
            false
        } else {
            for requirement in requirements {
                if self.points(&requirement.name) > 0 {
                    return true;
                }
            }
            false
        }
    }
}
impl SkillPoint for Decoration {
    fn points(&self, ability: &str) -> i32 {
        if ability == self.skill_1_name {
            self.skill_1_points
        } else if ability == self.skill_2_name {
            self.skill_2_points.unwrap_or(0)
        } else {
            0
        }
    }

    fn slots(&self) -> i32 {
        self.slots
    }


    fn translate_skills(&mut self, translation: &std::collections::HashMap<String, String>) {
        if let Some(t) = translation.get(&self.skill_1_name) {
            self.skill_1_name.clone_from(t);
        }
        if let Some(t) = translation.get(&self.skill_2_name) {
            self.skill_2_name.clone_from(t);
        }
    }

    fn max_defence(&self) -> u32 {
        0
    }
    fn defence(&self) -> u32 {
        0
    }
}
