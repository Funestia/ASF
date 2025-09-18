use std::collections::HashMap;


pub trait SkillPoint {
    fn points(&self, ability: &str) -> i32;
    fn max_defence(&self) -> u32;
    fn defence(&self) -> u32;
    fn slots(&self) -> i32;
    fn translate_skills(&mut self, translation: &HashMap<String, String>);
}
