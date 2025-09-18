use serde::{Deserialize, Serialize};

use crate::{requirements::Requirement, skillpoint::SkillPoint};

#[derive(Debug, PartialEq, Eq, Hash, Default, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct Charm {
    pub slots: i32,
    pub skill_1: String,
    pub points_1: i32,
    pub skill_2: String,
    pub points_2: Option<i32>,
}

impl SkillPoint for Charm {
    fn points(&self, ability: &str) -> i32 {
        if ability == self.skill_1 {
            self.points_1 
        } else if ability == self.skill_2 {
            self.points_2.unwrap_or(0)
        } else {
            0
        }
    }
    fn slots(&self) -> i32 {
        self.slots
    }

    fn translate_skills(&mut self, translation: &std::collections::HashMap<String, String>) {
        if let Some(t) = translation.get(&self.skill_1) {
            self.skill_1.clone_from(t);
        }
        if let Some(t) = translation.get(&self.skill_2) {
            self.skill_2.clone_from(t);
        }
    }

    fn max_defence(&self) -> u32 {
        0
    }

    fn defence(&self) -> u32 {
        0
    }
}
impl Charm {
    
    pub fn is_valid(&self, requirements: &[Requirement]) -> bool {
        if self.slots() == 3 {
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

impl std::fmt::Display for Charm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ret = write!(
            f,
            "{} {}",
            self.skill_1,
            self.points_1,
        );
        if let Some(points_2) = self.points_2 {
            write!(
                f,
                ", {} {}",
                self.skill_2,
                points_2
                )?;
        }
        for _ in 0..(self.slots) {
            write!(
                f,
                " ○"
        )?;
        }
        ret
    }
}
