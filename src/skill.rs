use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct Skill {
    pub name: String,
    pub name_attribute: String,
    pub points: i32,
    weapon_type: i32,
    pub category: Option<String>,
    pub category_2: Option<String>,
    pub max_weapon_skill_points: Option<i32>,
    pub max_armor_skill_points: Option<i32>
}

impl Skill {
    pub fn group(skills: &[Self]) -> (std::collections::HashMap<String, Vec<Skill>>, Vec<String>) {
        let skills_grouped = skills
            .iter()
            .cloned()
            .into_group_map_by(|skill| skill.name_attribute.clone());
        let mut skill_types = vec!["All".to_owned()];
        skill_types.extend(
            skills
                .iter()
                .filter_map(|skill| skill.category.clone())
                .filter(|s| s.parse::<i32>().is_err())
                .sorted()
                .unique(),
        );
        (skills_grouped, skill_types)
    }
    pub fn is_relic_skill(&self) -> bool {
        self.max_weapon_skill_points.is_some()
    }
    pub fn has_category(&self, category: Option<&str>) -> bool {
        if category == Some("All") || category == self.category.as_deref() {
            true
        } else if let Some(category_2) = &self.category_2 {
            Some(category_2.as_str()) == category
        } else {
            false
        }
    }
}
