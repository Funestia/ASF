use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    iter,
    mem::take,
};

use iced::{
    event, font,
    futures::{channel::mpsc, SinkExt, Stream},
    stream,
    widget::{
        button, checkbox, column, combo_box, container, horizontal_space, pick_list, progress_bar,
        row, scrollable, slider, text, text_input, Column, Row,
    },
    window::{self},
    Element, Event, Length, Padding, Task, Theme,
};
use iced_aw::number_input;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;

use crate::{
    algorithm::{find, scores, trim, FindResult},
    armor::Armor,
    charms::Charm,
    component::Component,
    decorations::Decoration,
    requirements::Requirement,
    segmented_button::segmented_button,
    skill::Skill,
    skillpoint::SkillPoint,
    types::{ArmorType, GatheringHallRank, Language, Sex, WeaponType},
};

const SCROLLBAR_WIDTH: u16 = 10;

fn contains_fuzzy(s: &str, pattern: &str) -> bool {
    let mut string_index = 0;
    'outer: for character in pattern.to_lowercase().chars() {
        while string_index < s.chars().count() {
            if Some(character) == s.to_lowercase().chars().nth(string_index) {
                continue 'outer;
            }
            string_index += 1;
        }
        return false;
    }
    true
}

#[derive(Debug, Clone)]
pub enum Message {
    Abort,
    EditCharms,
    EditRelics,
    PartExcludedRemoved(usize),
    CustomRemove(usize),
    CustomClear,
    CustomSave,
    CustomAdd,
    PartExcludedAdded(String),
    PartsExcludedClear,
    LanguageChanged(Language),
    ClearRequiredSkills,
    EventOccured(Event),
    SetsFound(FindResults),
    Search,
    SearchAdditionalSkills,
    GatheringHallRankSelected(GatheringHallRank),
    VillageRankSelected(i32),
    SexSelected(Sex),
    WeaponTypeSelected(WeaponType),
    WeaponSkillSelected(String),
    WeaponPointsSelected(i32),
    SkillFilterChanged(String),
    CustomSkillSetPoints(usize, i32),
    AvailableSkillSelected(String),
    NeededSkillChecked(usize, bool),
    SetNeededSkill(String, i32),
    NeededSkillDeleted(usize),
    NeededSkillIndexChanged(usize, usize),
    FontLoaded(Result<(), font::Error>),
    TrimCountChanged(f64),
    SetSlots(i32),
    SetProgress(f32),
    SetMinRarity(i32),
    SetProgressSender(mpsc::Sender<Option<f32>>),
    SetArmorName(String),
    SetArmorType(ArmorType),
}
struct Data {
    pub body: Vec<Armor>,
    pub head: Vec<Armor>,
    pub arms: Vec<Armor>,
    pub waist: Vec<Armor>,
    pub legs: Vec<Armor>,
    pub relic_body: Vec<Armor>,
    pub relic_head: Vec<Armor>,
    pub relic_arms: Vec<Armor>,
    pub relic_waist: Vec<Armor>,
    pub relic_legs: Vec<Armor>,
    pub charms: Vec<Charm>,
    pub decorations: Vec<Decoration>,
    pub skills: Vec<Skill>,
}

impl Data {
    fn translate(language: Language, flags: &Flags) -> Data {
        let (attribute_translation_map, skills) = {
            let mut skills = flags.skills.clone();
            let mut skill_attributes = Vec::new();
            let mut seperator = false;
            let mut skill_start_index = 0;
            for (pos, skill_translation) in
                BufReader::new(File::open(format!("./Languages/{language}/skills.txt")).unwrap())
                    .lines()
                    .enumerate()
            {
                if seperator {
                    skills[pos - skill_start_index].name = skill_translation.unwrap();
                } else if skill_translation.as_ref().is_ok_and(|x| !x.is_empty()) {
                    skill_attributes.push(skill_translation.unwrap());
                } else {
                    seperator = true;
                    skill_start_index = pos + 1;
                }
            }
            let mut attribute_translation_map = HashMap::new();
            for (attribute, translation) in skills
                .iter_mut()
                .map(|s| s.name_attribute.clone())
                .unique()
                .zip(skill_attributes.into_iter())
            {
                attribute_translation_map.insert(attribute, translation);
            }
            for skill in skills.iter_mut() {
                skill
                    .name_attribute
                    .clone_from(&attribute_translation_map[&skill.name_attribute]);
            }
            (attribute_translation_map, skills)
        };
        let helper_function = |parts: &[Armor], part_name| {
            parts
                .iter()
                .cloned()
                .zip(
                    BufReader::new(
                        File::open(format!("./Languages/{language}/{part_name}.txt")).unwrap(),
                    )
                    .lines(),
                )
                .map(|(mut x, translation)| {
                    x.translate_skills(&attribute_translation_map);
                    x.name = translation.unwrap_or_default();
                    x
                })
                .collect_vec()
        };
        let head = helper_function(&flags.head, "head");
        let arms = helper_function(&flags.arms, "arms");
        let waist = helper_function(&flags.waist, "waist");
        let legs = helper_function(&flags.legs, "legs");
        let body = helper_function(&flags.body, "body");
        let charms = flags
            .charms
            .clone()
            .into_iter()
            .map(|mut x| {
                x.translate_skills(&attribute_translation_map);
                x
            })
            .collect_vec();
        let relic_head = flags.relic_head.clone();
        let relic_arms = flags.relic_arms.clone();
        let relic_waist = flags.relic_waist.clone();
        let relic_legs = flags.relic_legs.clone();
        let relic_body = flags.relic_body.clone();
        let decorations = flags
            .decorations
            .clone()
            .into_iter()
            .zip(
                BufReader::new(
                    File::open(format!("./Languages/{language}/decorations.txt")).unwrap(),
                )
                .lines(),
            )
            .map(|(mut x, translation)| {
                x.translate_skills(&attribute_translation_map);
                x.name = translation.unwrap_or_default();
                x
            })
            .collect_vec();
        Data {
            skills,
            head,
            body,
            arms,
            waist,
            legs,
            relic_body,
            relic_legs,
            relic_waist,
            relic_arms,
            relic_head,
            decorations,
            charms,
        }
    }
}
#[derive(Clone)]
pub struct Flags {
    pub components: Vec<Component>,
    pub body: Vec<Armor>,
    pub head: Vec<Armor>,
    pub arms: Vec<Armor>,
    pub waist: Vec<Armor>,
    pub legs: Vec<Armor>,
    pub relic_body: Vec<Armor>,
    pub relic_head: Vec<Armor>,
    pub relic_arms: Vec<Armor>,
    pub relic_waist: Vec<Armor>,
    pub relic_legs: Vec<Armor>,
    pub charms: Vec<Charm>,
    pub decorations: Vec<Decoration>,
    pub skills: Vec<Skill>,
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    gathering_hall_rank: GatheringHallRank,
    language: Language,
    village_rank: i32,
    weapon_type: WeaponType,
    weapon_slots: usize,
    weapon_skill: Option<Requirement>,
    sex: Sex,
    skill_type_index: usize,
    skill_filter: String,
    skills_needed: Vec<(bool, usize, Vec<Skill>)>,
    parts_excluded: Vec<String>,
    trim_count: usize,
    min_rarity: i32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            language: Language::German,
            gathering_hall_rank: GatheringHallRank::All,
            village_rank: 10,
            weapon_type: WeaponType::Melee,
            weapon_slots: 0,
            sex: Sex::Male,
            skill_type_index: 0,
            min_rarity: 1,
            skill_filter: String::default(),
            skills_needed: Vec::default(),
            parts_excluded: Vec::default(),
            trim_count: 20,
            weapon_skill: Default::default(),
        }
    }
}
enum SearchStatus<T> {
    Found(Vec<T>),
    Searching(f32),
}

impl<T> Default for SearchStatus<T> {
    fn default() -> Self {
        SearchStatus::Found(Vec::new())
    }
}

#[derive(Clone, Debug)]
pub enum FindResults {
    ArmorSets(Vec<FindResult>),
    Skills(Vec<usize>),
}

struct CharmsState {
    pub charms: Vec<Charm>,
    pub skills: Vec<Requirement>,
    pub slots: i32,
}

enum RelicArmorSkill {
    Slots(i32),
    Skill(Requirement),
}
impl Default for RelicArmorSkill {
    fn default() -> Self {
        RelicArmorSkill::Slots(0)
    }
}
impl RelicArmorSkill {
    pub fn get_skill_name(&self) -> &str {
        match &self {
            RelicArmorSkill::Slots(_) => "",
            RelicArmorSkill::Skill(requirement) => &requirement.name,
        }
    }
    pub fn is_empty(&self) -> bool {
        match &self {
            RelicArmorSkill::Slots(x) => *x == 0,
            RelicArmorSkill::Skill(requirement) => requirement.points == 0,
        }
    }
    pub fn slots(&self) -> i32 {
        match self {
            RelicArmorSkill::Slots(x) => *x,
            RelicArmorSkill::Skill(_requirement) => 0,
        }
    }
    pub fn points(&self) -> i32 {
        match self {
            RelicArmorSkill::Slots(_) => 0,
            RelicArmorSkill::Skill(requirement) => requirement.points,
        }
    }
}
#[derive(Default)]
struct ArmorState {
    pub pieces: Vec<(ArmorType, Armor)>,
    pub skill: RelicArmorSkill,
    pub weapon_type: WeaponType,
    pub armor_type: ArmorType,
    pub name: String,
}

impl ArmorState {
    pub fn piece_to_string(piece: &(ArmorType, Armor)) -> String {
        let (_t, p) = piece;
        let mut result = p.name.clone();
        if let Some(points) = p.ability_1_points {
            result.push_str(&format!(" {} {points}", p.ability_1_name));
        }
        for _ in 0..(p.slots) {
            result.push_str(" ○");
        }
        result
    }
}

#[derive(Default)]
enum UIState {
    #[default]
    Default,
    Charms(CharmsState),
    Armor(ArmorState),
}
pub struct State {
    pub weapon_skills: combo_box::State<String>,
    data: Data,
    ui_state: UIState,
    flags: Flags,
    settings: Settings,
    search_status: SearchStatus<FindResult>,
    search_status_skills: SearchStatus<usize>,
    skill_types: Vec<String>,
    progress_sender: Option<mpsc::Sender<Option<f32>>>,
    skills_grouped: HashMap<String, Vec<Skill>>,
}
impl State {
    pub fn new(flags: Flags) -> Self {
        let settings: Settings = if let Ok(file) = File::open("./settings") {
            bincode::deserialize_from(file).unwrap_or_default()
        } else {
            Default::default()
        };
        let data = Data::translate(settings.language, &flags);
        let (skills_grouped, skill_types) = Skill::group(&data.skills);
        let weapon_skills = combo_box::State::new(
            skills_grouped
                .iter()
                .filter_map(|(k, v)| {
                    v[0].max_weapon_skill_points
                        .is_some_and(|x| x > 0)
                        .then_some(k.clone())
                })
                .collect(),
        );
        State {
            weapon_skills,
            data,
            flags,
            skills_grouped,
            skill_types,
            settings,
            progress_sender: None,
            search_status: Default::default(),
            search_status_skills: Default::default(),
            ui_state: Default::default(),
        }
    }
}
fn progress_worker() -> impl Stream<Item = Message> {
    stream::channel(100, |mut output| async move {
        let (sender, mut receiver) = mpsc::channel(100);
        output
            .send(Message::SetProgressSender(sender))
            .await
            .unwrap();
        loop {
            use iced::futures::StreamExt;
            if let Some(input) = receiver.select_next_some().await {
                output.send(Message::SetProgress(input)).await.unwrap();
            }
        }
    })
}
pub fn subscription(_state: &State) -> iced::Subscription<Message> {
    iced::Subscription::batch([
        event::listen().map(Message::EventOccured),
        iced::Subscription::run(progress_worker),
    ])
}

pub fn update(appstate: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Abort => {
            appstate.ui_state = UIState::Default;
            Task::none()
        }
        Message::SetMinRarity(rarity) => {
            appstate.settings.min_rarity = rarity;
            Task::none()
        }
        Message::CustomClear => {
            if let UIState::Charms(ref mut state) = appstate.ui_state {
                state.skills.clear();
                state.slots = 0;
            }
            Task::none()
        }
        Message::SetSlots(slots) => {
            match appstate.ui_state {
                UIState::Charms(ref mut state) => state.slots = slots,
                UIState::Armor(ref mut state) => state.skill = RelicArmorSkill::Slots(slots),
                UIState::Default => appstate.settings.weapon_slots = slots as usize,
            }
            Task::none()
        }
        Message::CustomSave => {
            match appstate.ui_state {
                UIState::Charms(ref mut state) => {
                    // let mut data_charms = appstate.data.charms.unwrap();
                    let mut writer = csv::WriterBuilder::new()
                        .has_headers(false)
                        .from_path("./mycharms.txt")
                        .unwrap();
                    writer.serialize(Charm::default()).unwrap();
                    for charm in &state.charms {
                        writer.serialize(charm).unwrap();
                    }
                    appstate.data.charms = take(&mut state.charms);
                    appstate.ui_state = UIState::Default;
                }
                UIState::Armor(ref mut state) => {
                    let helper = |data_custom: &mut Vec<Armor>, path: &str, t: ArmorType| {
                        let mut writer = csv::WriterBuilder::new()
                            .has_headers(false)
                            .from_path(path)
                            .unwrap();
                        data_custom.clear();
                        for (_t, piece) in state.pieces.iter().filter(|(x, _p)| t == *x) {
                            writer.serialize(piece).unwrap();
                            data_custom.push(piece.clone());
                        }
                    };
                    helper(
                        &mut appstate.data.relic_head,
                        "./relic_head.txt",
                        ArmorType::Head,
                    );
                    helper(
                        &mut appstate.data.relic_arms,
                        "./relic_arms.txt",
                        ArmorType::Arms,
                    );
                    helper(
                        &mut appstate.data.relic_body,
                        "./relic_chest.txt",
                        ArmorType::Chest,
                    );
                    helper(
                        &mut appstate.data.relic_waist,
                        "./relic_waist.txt",
                        ArmorType::Waist,
                    );
                    helper(
                        &mut appstate.data.relic_legs,
                        "./relic_legs.txt",
                        ArmorType::Legs,
                    );
                    appstate.ui_state = UIState::Default;
                }
                _ => (),
            }
            Task::none()
        }
        Message::CustomRemove(index) => {
            match appstate.ui_state {
                UIState::Charms(ref mut state) => {
                    state.charms.remove(index);
                }
                UIState::Armor(ref mut state) => {
                    state.pieces.remove(index);
                }
                _ => (),
            }
            Task::none()
        }
        Message::CustomAdd => {
            match appstate.ui_state {
                UIState::Charms(ref mut state) => {
                    if let Some(req_1) = state.skills.first() {
                        let req_2 = state.skills.get(1);
                        state.charms.push(Charm {
                            slots: state.slots,
                            skill_1: req_1.name.clone(),
                            points_1: req_1.points,
                            points_2: req_2.map(|r| r.points),
                            skill_2: req_2.map_or(String::new(), |r| r.name.clone()),
                        });
                    }
                }
                UIState::Armor(ref mut state) => {
                    let piece = Armor {
                        name: format!("🪙 {} {}", state.name, &state.skill.get_skill_name()[..4]),
                        weapon_type: take(&mut state.weapon_type) as i32,
                        slots: state.skill.slots(),
                        rarity: 10,
                        ability_1_name: state.skill.get_skill_name().to_string(),
                        ability_1_points: Some(state.skill.points()),
                        ..Default::default()
                    };
                    state.pieces.push((state.armor_type, piece));
                    state.name = Default::default();
                    state.skill = Default::default();
                }
                _ => (),
            }
            update(appstate, Message::CustomClear)
        }
        Message::CustomSkillSetPoints(index, points) => {
            match appstate.ui_state {
                UIState::Charms(ref mut state) => {
                    if let Some(skill) = state.skills.get_mut(index) {
                        skill.points = points;
                    }
                }
                UIState::Armor(ref mut state) => {
                    if let RelicArmorSkill::Skill(Requirement {
                        name: _,
                        points: ref mut skillpoints,
                    }) = state.skill
                    {
                        *skillpoints = points;
                    }
                }
                _ => (),
            }
            Task::none()
        }
        Message::EditCharms => {
            appstate.ui_state = UIState::Charms(CharmsState {
                charms: appstate.data.charms.clone(),
                skills: Default::default(),
                slots: 0,
            });
            Task::none()
        }
        Message::PartsExcludedClear => {
            appstate.settings.parts_excluded.clear();
            Task::none()
        }
        Message::PartExcludedRemoved(idx) => {
            appstate.settings.parts_excluded.remove(idx);
            Task::none()
        }
        Message::PartExcludedAdded(part) => {
            appstate.settings.parts_excluded.push(part);
            update(appstate, Message::Search)
        }
        Message::LanguageChanged(language) => {
            appstate.data = Data::translate(language, &appstate.flags);
            (appstate.skills_grouped, appstate.skill_types) = Skill::group(&appstate.data.skills);
            appstate.settings.language = language;
            update(appstate, Message::ClearRequiredSkills)
        }
        Message::ClearRequiredSkills => {
            match appstate.ui_state {
                UIState::Default => appstate.settings.skills_needed.clear(),
                UIState::Charms(ref mut state) => state.skills.clear(),
                UIState::Armor(ref mut state) => state.skill = RelicArmorSkill::Slots(0),
            };
            Task::none()
        }
        Message::EventOccured(event) => {
            if Event::Window(window::Event::CloseRequested) == event {
                bincode::serialize_into(File::create("./settings").unwrap(), &appstate.settings)
                    .unwrap();
                window::get_oldest().then(|id| window::close(id.unwrap()))
            } else {
                Task::none()
            }
        }
        Message::TrimCountChanged(trim_count) => {
            appstate.settings.trim_count = trim_count as usize;
            Task::none()
        }
        Message::NeededSkillIndexChanged(index, skillindex) => {
            appstate.settings.skills_needed[index].1 = skillindex;
            Task::none()
        }
        Message::NeededSkillDeleted(index) => {
            match appstate.ui_state {
                UIState::Default => {
                    appstate.settings.skills_needed.remove(index);
                }
                UIState::Charms(ref mut state) => {
                    state.skills.remove(index);
                }
                UIState::Armor(ref mut state) => {
                    state.skill = RelicArmorSkill::Slots(0);
                }
            };
            Task::none()
        }
        Message::NeededSkillChecked(index, value) => {
            appstate.settings.skills_needed[index].0 = value;
            Task::none()
        }
        Message::AvailableSkillSelected(name) => {
            match appstate.ui_state {
                UIState::Default => {
                    let skill = appstate.skills_grouped.get(&name).unwrap().clone();
                    appstate.settings.skills_needed.push((true, 0, skill));
                }
                UIState::Charms(ref mut state) => {
                    state.skills.push(Requirement { name, points: 0 });
                }
                UIState::Armor(ref mut state) => {
                    state.skill = RelicArmorSkill::Skill(Requirement { name, points: 0 });
                }
            }
            Task::none()
        }
        Message::SetNeededSkill(name, points) => {
            if let Some((active, index, skills)) = appstate
                .settings
                .skills_needed
                .iter_mut()
                .find(|g| g.2[0].name_attribute == name)
            {
                *active = true;
                if let Some((idx, _)) = skills.iter().enumerate().find(|(_, x)| x.points == points)
                {
                    *index = idx;
                }
            } else {
                let skill = appstate.skills_grouped.get(&name).unwrap().clone();
                let mut index = 0;
                if let Some((idx, _)) = skill.iter().enumerate().find(|(_, x)| x.points == points) {
                    index = idx;
                }
                appstate.settings.skills_needed.push((true, index, skill));
            }
            Task::none()
        }
        Message::SkillFilterChanged(filter) => {
            appstate.settings.skill_filter = filter;
            Task::none()
        }
        Message::SexSelected(sex) => {
            appstate.settings.sex = sex;
            Task::none()
        }
        Message::WeaponTypeSelected(weapon_type) => {
            match appstate.ui_state {
                UIState::Default => appstate.settings.weapon_type = weapon_type,
                UIState::Armor(ref mut armor_state) => armor_state.weapon_type = weapon_type,
                _ => (),
            }
            Task::none()
        }
        Message::WeaponSkillSelected(name) => {
            appstate.settings.weapon_skill = Some(Requirement { name, points: 0 });
            Task::none()
        }
        Message::WeaponPointsSelected(points) => {
            if let Some(ref mut skill) = appstate.settings.weapon_skill {
                skill.points = points;
            }
            Task::none()
        }
        Message::VillageRankSelected(rank) => {
            appstate.settings.village_rank = rank;
            Task::none()
        }
        Message::GatheringHallRankSelected(rank) => {
            appstate.settings.gathering_hall_rank = rank;
            Task::none()
        }
        Message::SetsFound(results) => {
            match results {
                FindResults::ArmorSets(sets) => appstate.search_status = SearchStatus::Found(sets),
                FindResults::Skills(skills) => {
                    appstate.search_status_skills = SearchStatus::Found(skills)
                }
            }
            Task::none()
        }
        msg @ (Message::Search | Message::SearchAdditionalSkills) => {
            match msg {
                Message::Search => appstate.search_status = SearchStatus::Searching(0.0),
                _ => appstate.search_status_skills = SearchStatus::Searching(0.0),
            }
            let mut head = appstate.data.head.clone();
            head.extend_from_slice(&appstate.data.relic_head);
            let mut body = appstate.data.body.clone();
            body.extend_from_slice(&appstate.data.relic_body);
            let mut waist = appstate.data.waist.clone();
            waist.extend_from_slice(&appstate.data.relic_waist);
            let mut legs = appstate.data.legs.clone();
            legs.extend_from_slice(&appstate.data.relic_legs);
            let mut arms = appstate.data.arms.clone();
            arms.extend_from_slice(&appstate.data.relic_arms);
            let charms = appstate.data.charms.clone();
            let decorations = appstate.data.decorations.clone();
            let weapon_slots = appstate.settings.weapon_slots;
            let skills = appstate.data.skills.clone();
            let weapon_skill_name = appstate
                .settings
                .weapon_skill
                .as_ref()
                .map(|x| x.name.clone())
                .unwrap_or_default();
            let requirements = appstate
                .settings
                .skills_needed
                .iter()
                .filter_map(|(active, index, skills)| {
                    active.then_some(Requirement {
                        name: skills[*index].name_attribute.clone(),
                        points: skills[*index].points
                            - if skills[*index].name_attribute == weapon_skill_name {
                                appstate.settings.weapon_skill.as_ref().unwrap().points
                            } else {
                                0
                            },
                    })
                })
                .collect_vec();
            let village_rank = appstate.settings.village_rank;
            let sex = appstate.settings.sex as i32;
            let weapon_type = appstate.settings.weapon_type as i32;
            let gathering_hall_rank = appstate.settings.gathering_hall_rank as i32;
            let trim_count = appstate.settings.trim_count;
            let language = appstate.settings.language;
            let components = appstate.flags.components.clone();
            let excluded = appstate.settings.parts_excluded.clone();
            let mut progress_sender = appstate.progress_sender.clone().unwrap();
            let min_rarity = appstate.settings.min_rarity;
            Task::perform(
                async move {
                    let mut additional_skills = Vec::new();
                    let mut requirements_mut = requirements.clone();
                    for (index, skill) in skills.iter().enumerate().filter(|(_, skill)| {
                        skill.points > 0
                            && requirements.iter().all(|req| {
                                req.name != skill.name_attribute || req.points < skill.points
                            })
                    }) {
                        if matches!(msg, Message::SearchAdditionalSkills) {
                            requirements_mut.push(Requirement {
                                name: skill.name_attribute.clone(),
                                points: skill.points,
                            });
                        }
                        let mut progress_sender = progress_sender.clone();
                        let decorations: (Vec<usize>, Vec<&Decoration>) = decorations
                            .iter()
                            .enumerate()
                            .filter(|(_, d)| {
                                d.is_valid(
                                    gathering_hall_rank,
                                    village_rank,
                                    language == Language::Japanese,
                                    &components,
                                    &requirements_mut,
                                ) && !excluded.contains(&d.name)
                            })
                            .unzip();
                        let scores = scores(
                            &requirements_mut,
                            &decorations.1.clone().into_iter().cloned().collect_vec(),
                        );
                        let helper_function = |part| {
                            trim(
                                part,
                                |p: &Armor| {
                                    p.is_valid(
                                        gathering_hall_rank,
                                        village_rank,
                                        min_rarity,
                                        sex,
                                        language == Language::Japanese,
                                        weapon_type,
                                        &components,
                                        &requirements_mut,
                                    ) && !excluded.contains(&p.name)
                                },
                                &requirements_mut,
                                &scores,
                                trim_count,
                            )
                        };
                        let head = helper_function(&head);
                        let body = helper_function(&body);
                        let arms = helper_function(&arms);
                        let waist = helper_function(&waist);
                        let legs = helper_function(&legs);
                        let charms = trim(
                            &charms,
                            |charm| charm.is_valid(&requirements_mut),
                            &requirements_mut,
                            &scores,
                            trim_count,
                        );
                        let mut results = find(
                            &head.1,
                            &body.1,
                            &arms.1,
                            &waist.1,
                            &legs.1,
                            &charms.1,
                            &decorations.1,
                            &requirements_mut,
                            weapon_slots,
                            if matches!(msg, Message::SearchAdditionalSkills) {
                                1
                            } else {
                                200
                            },
                        );
                        for result in results.iter_mut() {
                            result.head_index =
                                result.head_index.and_then(|i| head.0.get(i).cloned());
                            result.arms_index =
                                result.arms_index.and_then(|i| arms.0.get(i).cloned());
                            result.waist_index =
                                result.waist_index.and_then(|i| waist.0.get(i).cloned());
                            result.legs_index =
                                result.legs_index.and_then(|i| legs.0.get(i).cloned());
                            result.charms_index = charms.0[result.charms_index];
                            result.body_index = body.0[result.body_index];
                            for (_, decoration_index) in result.decorations_count_indices.iter_mut()
                            {
                                *decoration_index = decorations.0[*decoration_index];
                            }
                        }
                        if matches!(msg, Message::Search) {
                            return FindResults::ArmorSets(results);
                        } else if !results.is_empty() {
                            additional_skills.push(index);
                        }
                        requirements_mut.pop();
                        let progress = (100.0 * index as f32) / skills.len() as f32;
                        progress_sender.try_send(Some(progress)).unwrap();
                    }
                    progress_sender.try_send(None).unwrap();
                    FindResults::Skills(additional_skills)
                },
                Message::SetsFound,
            )
        }
        Message::FontLoaded(_) => Task::none(),
        Message::SetProgress(progress) => {
            appstate.search_status_skills = SearchStatus::Searching(progress);
            Task::none()
        }
        Message::SetProgressSender(sender) => {
            appstate.progress_sender = Some(sender);
            Task::none()
        }
        Message::EditRelics => {
            let head = iter::repeat(ArmorType::Head).zip(appstate.data.relic_head.clone());
            let chest = iter::repeat(ArmorType::Chest).zip(appstate.data.relic_body.clone());
            let arms = iter::repeat(ArmorType::Arms).zip(appstate.data.relic_arms.clone());
            let waist = iter::repeat(ArmorType::Waist).zip(appstate.data.relic_waist.clone());
            let legs = iter::repeat(ArmorType::Legs).zip(appstate.data.relic_legs.clone());
            let pieces = head
                .chain(chest)
                .chain(arms)
                .chain(waist)
                .chain(legs)
                .collect();
            appstate.ui_state = UIState::Armor(ArmorState {
                pieces,
                skill: RelicArmorSkill::Slots(0),
                weapon_type: WeaponType::Both,
                armor_type: ArmorType::default(),
                name: "".to_owned(),
            });
            Task::none()
        }
        Message::SetArmorName(name) => {
            if let UIState::Armor(ref mut state) = appstate.ui_state {
                state.name = name;
            }
            Task::none()
        }
        Message::SetArmorType(armor_type) => {
            if let UIState::Armor(ref mut state) = appstate.ui_state {
                state.armor_type = armor_type;
            }
            Task::none()
        }
    }
}

pub fn view(appstate: &State) -> iced::Element<'_, Message, Theme> {
    row![
        match &appstate.ui_state {
            //general stuff aka rank, type, sex, etc
            UIState::Default => column![
                row![
                    text("Language"),
                    horizontal_space(),
                    pick_list(
                        Language::all(),
                        Some(appstate.settings.language),
                        Message::LanguageChanged
                    )
                    .text_shaping(text::Shaping::Advanced),
                ]
                .width(Length::Fixed(230f32)),
                row![
                    text("Gathering Hall"),
                    horizontal_space(),
                    pick_list(
                        GatheringHallRank::all(),
                        Some(appstate.settings.gathering_hall_rank),
                        Message::GatheringHallRankSelected
                    ),
                ]
                .width(Length::Fill),
                row![
                    text("Village Rank"),
                    horizontal_space(),
                    pick_list(
                        (1..=10).collect_vec(),
                        Some(appstate.settings.village_rank),
                        Message::VillageRankSelected
                    ),
                ],
                row![
                    text("Minimum Armor Rarity"),
                    horizontal_space(),
                    pick_list(
                        (1..=10).collect_vec(),
                        Some(appstate.settings.min_rarity),
                        Message::SetMinRarity
                    ),
                ],
                row![
                    text("Max Weapon Slots"),
                    horizontal_space(),
                    pick_list(
                        (0..=3).collect_vec(),
                        Some(appstate.settings.weapon_slots as i32),
                        Message::SetSlots
                    ),
                ],
                text("Weapon Skill"),
                {
                    let row = Row::with_children([combo_box(
                        &appstate.weapon_skills,
                        "",
                        appstate.settings.weapon_skill.as_ref().map(|x| &x.name),
                        Message::WeaponSkillSelected,
                    )
                    .into()]);
                    if let Some(Requirement { name: _, points }) = &appstate.settings.weapon_skill {
                        row.push(number_input(points, 0..=6, Message::WeaponPointsSelected))
                    } else {
                        row
                    }
                },
                text(""),
                row![
                    segmented_button(
                        text(Sex::Male.to_string()),
                        Sex::Male,
                        Some(appstate.settings.sex),
                        Message::SexSelected
                    )
                    .width(Length::Fill),
                    segmented_button(
                        text(Sex::Female.to_string()),
                        Sex::Female,
                        Some(appstate.settings.sex),
                        Message::SexSelected
                    )
                    .width(Length::Fill),
                ],
                row![
                    segmented_button(
                        text(WeaponType::Melee.to_string()),
                        WeaponType::Melee,
                        Some(appstate.settings.weapon_type),
                        Message::WeaponTypeSelected
                    )
                    .width(Length::Fill),
                    segmented_button(
                        text(WeaponType::Marksman.to_string()),
                        WeaponType::Marksman,
                        Some(appstate.settings.weapon_type),
                        Message::WeaponTypeSelected
                    )
                    .width(Length::Fill),
                ],
                text(format!(
                    "Considered Parts per Slot: {}",
                    appstate.settings.trim_count
                )),
                slider(
                    15.0..=60.0,
                    appstate.settings.trim_count as f64,
                    Message::TrimCountChanged
                ),
                text(""),
                button(text("Edit Charms")).on_press(Message::EditCharms),
                button(text("Edit Relics")).on_press(Message::EditRelics),
                text(""),
                if !appstate.settings.parts_excluded.is_empty() {
                    column![
                        row![
                            text("Excluded parts\n").width(Length::Fill),
                            button(text("🗑️").shaping(text::Shaping::Advanced))
                                .on_press(Message::PartsExcludedClear)
                                .style(button::danger),
                        ],
                        scrollable(appstate.settings.parts_excluded.iter().enumerate().fold(
                            Column::new().padding(Padding::ZERO.right(SCROLLBAR_WIDTH)),
                            |col, (index, name)| {
                                col.push(row![
                                    text(name).width(Length::Fill),
                                    button(text("🗑️").shaping(text::Shaping::Advanced))
                                        .on_press(Message::PartExcludedRemoved(index))
                                        .style(button::danger),
                                ])
                            },
                        )),
                    ]
                } else {
                    column![]
                }
            ]
            .width(Length::Shrink),
            _ => column![],
        },
        column![
            text("Available Skills"),
            text_input("search", &appstate.settings.skill_filter)
                .on_input(Message::SkillFilterChanged)
                .width(Length::Fixed(200f32)),
            scrollable(
                appstate
                    .skills_grouped
                    .iter()
                    .filter(|(name, skills)| match appstate.ui_state {
                        UIState::Default => appstate
                            .settings
                            .skills_needed
                            .iter()
                            .all(|(_, _, skills)| &&skills[0].name_attribute != name),
                        UIState::Charms(ref state) =>
                            state.skills.iter().all(|req| &&req.name != name),
                        UIState::Armor(ref state) =>
                            &state.skill.get_skill_name() != name && skills[0].is_relic_skill(),
                    })
                    .filter_map(|(name, skills)| {
                        let text = format!("{name} {}", {
                            let mut buffer = String::new();
                            for skill in skills {
                                if skill.points > 0 {
                                    write!(&mut buffer, " {}", skill.name).unwrap()
                                };
                            }
                            buffer
                        });
                        contains_fuzzy(&text, &appstate.settings.skill_filter)
                            .then_some((name, text))
                    })
                    .fold(
                        Column::new().padding(Padding::ZERO.right(SCROLLBAR_WIDTH)),
                        |col, (name, button_text)| col.push(
                            button(text(button_text))
                                .on_press(Message::AvailableSkillSelected(name.clone()))
                                .style(button::secondary)
                                .width(Length::Fill)
                        )
                    )
            )
        ]
        .width(Length::FillPortion(5)),
        match appstate.ui_state {
            UIState::Default => column![
                row![
                    text("Required skills").width(Length::Fill),
                    button("clear").on_press(Message::ClearRequiredSkills)
                ]
                .align_y(iced::Alignment::Center),
                scrollable(
                    Column::with_children(appstate.settings.skills_needed.iter().enumerate().map(
                        |(index, (active, active_skill_index, skills))| {
                            row![
                                checkbox("", *active)
                                    .on_toggle(move |a| Message::NeededSkillChecked(index, a)),
                                Row::with_children(
                                    skills
                                        .iter()
                                        .enumerate()
                                        .filter(|(_, skill)| skill.points > 0)
                                        .map(|(skill_index, skill)| segmented_button(
                                            text(&skill.name),
                                            skill_index,
                                            Some(*active_skill_index),
                                            |x| Message::NeededSkillIndexChanged(index, x)
                                        )
                                        .width(Length::Fill)
                                        .into())
                                        .collect_vec()
                                )
                                .align_y(iced::Alignment::Center)
                                .height(Length::Fill),
                                button(text("🗑️").shaping(text::Shaping::Advanced))
                                    .on_press(Message::NeededSkillDeleted(index))
                                    .style(button::danger),
                            ]
                            .align_y(iced::Alignment::Center)
                            .height(Length::Shrink)
                            .into()
                        }
                    ))
                    .padding(Padding::ZERO.right(SCROLLBAR_WIDTH))
                ),
                text(""),
                row![
                    text("Possible additional skills").width(Length::Fill),
                    match &appstate.search_status_skills {
                        SearchStatus::Found(_) =>
                            Element::from(button("find").on_press(Message::SearchAdditionalSkills)),
                        SearchStatus::Searching(progress) =>
                            progress_bar(0.0..=100.0, *progress).into(),
                    }
                ],
                match &appstate.search_status_skills {
                    SearchStatus::Found(results) =>
                        if results.is_empty() {
                            column![text("None")].align_x(iced::Alignment::Center)
                        } else {
                            column![scrollable(
                                Column::with_children(results.iter().map(|&result| {
                                    let skill = &appstate.data.skills[result];
                                    button(text(skill.name.clone()))
                                        .on_press(Message::SetNeededSkill(
                                            skill.name_attribute.clone(),
                                            skill.points,
                                        ))
                                        .style(button::secondary)
                                        .width(Length::Fill)
                                        .into()
                                }))
                                .padding(Padding::ZERO.right(SCROLLBAR_WIDTH)),
                            )
                            .width(Length::Fill)]
                        },
                    SearchStatus::Searching(_) =>
                        column![text("Searching...")].align_x(iced::Alignment::Center),
                }
            ]
            .width(Length::FillPortion(6)),
            UIState::Charms(ref state) => column![
                row![
                    text("Slots"),
                    number_input(&state.slots, 0..=3, Message::SetSlots)
                ],
                text(""),
                row![
                    text("Skills").width(Length::Fill),
                    button("clear").on_press(Message::ClearRequiredSkills)
                ],
                Column::with_children(state.skills.iter().enumerate().map(|(index, req)| {
                    row![
                        number_input(&req.points, -20..=20, move |x| {
                            Message::CustomSkillSetPoints(index, x)
                        }),
                        text(req.name.clone()).width(Length::Fill),
                        button(text("🗑️").shaping(text::Shaping::Advanced))
                            .on_press(Message::NeededSkillDeleted(index))
                            .style(button::danger),
                    ]
                    .into()
                })),
            ]
            .width(Length::FillPortion(4)),
            UIState::Armor(ref state) => column![
                row![
                    text("Name"),
                    horizontal_space(),
                    text_input("Some Piece", &state.name).on_input(Message::SetArmorName)
                ],
                Row::with_children(ArmorType::all().iter().map(|&t| {
                    segmented_button(
                        text(t.to_string()),
                        t,
                        Some(state.armor_type),
                        Message::SetArmorType,
                    )
                    .into()
                })),
                row![
                    segmented_button(
                        text(WeaponType::Both.to_string()),
                        WeaponType::Both,
                        Some(state.weapon_type),
                        Message::WeaponTypeSelected
                    ),
                    segmented_button(
                        text(WeaponType::Melee.to_string()),
                        WeaponType::Melee,
                        Some(state.weapon_type),
                        Message::WeaponTypeSelected
                    ),
                    segmented_button(
                        text(WeaponType::Marksman.to_string()),
                        WeaponType::Marksman,
                        Some(state.weapon_type),
                        Message::WeaponTypeSelected
                    ),
                ],
                match &state.skill {
                    RelicArmorSkill::Slots(slots) => row![
                        number_input(slots, 0..=3, Message::SetSlots),
                        text("Slots").width(Length::Fill)
                    ],
                    RelicArmorSkill::Skill(requirement) => row![
                        number_input(&requirement.points, 0..=6, |x| {
                            Message::CustomSkillSetPoints(0, x)
                        }),
                        text(&requirement.name).width(Length::Fill),
                        button(text("🗑️").shaping(text::Shaping::Advanced))
                            .on_press(Message::NeededSkillDeleted(0))
                            .style(button::danger),
                    ],
                }
            ]
            .width(Length::FillPortion(4)),
        },
        match appstate.ui_state {
            UIState::Default => match &appstate.search_status {
                SearchStatus::Found(results) => column![
                    button(text("find")).on_press(Message::Search),
                    text(format!("{} results", results.len())),
                    //maybe add sorting
                    scrollable(
                        Column::with_children(
                            results
                                .iter()
                                .map(|result| {
                                    let torso_up_count = [
                                        result.head_index,
                                        result.waist_index,
                                        result.arms_index,
                                        result.legs_index,
                                    ]
                                    .iter()
                                    .filter(|x| x.is_none())
                                    .count();
                                    let head =
                                        result.head_index.map_or("torso up".to_owned(), |index| {
                                            appstate
                                                .data
                                                .head
                                                .get(index)
                                                .unwrap_or_else(|| {
                                                    &appstate.data.relic_head
                                                        [index - appstate.data.head.len()]
                                                })
                                                .name
                                                .clone()
                                        });
                                    let waist =
                                        result.waist_index.map_or("torso up".to_owned(), |index| {
                                            appstate
                                                .data
                                                .waist
                                                .get(index)
                                                .unwrap_or_else(|| {
                                                    &appstate.data.relic_waist
                                                        [index - appstate.data.waist.len()]
                                                })
                                                .name
                                                .clone()
                                        });
                                    let arms =
                                        result.arms_index.map_or("torso up".to_owned(), |index| {
                                            appstate
                                                .data
                                                .arms
                                                .get(index)
                                                .unwrap_or_else(|| {
                                                    &appstate.data.relic_arms
                                                        [index - appstate.data.arms.len()]
                                                })
                                                .name
                                                .clone()
                                        });
                                    let legs =
                                        result.legs_index.map_or("torso up".to_owned(), |index| {
                                            appstate
                                                .data
                                                .legs
                                                .get(index)
                                                .unwrap_or_else(|| {
                                                    &appstate.data.relic_legs
                                                        [index - appstate.data.legs.len()]
                                                })
                                                .name
                                                .clone()
                                        });
                                    let body = appstate
                                        .data
                                        .body
                                        .get(result.body_index)
                                        .unwrap_or_else(|| {
                                            &appstate.data.relic_body
                                                [result.body_index - appstate.data.body.len()]
                                        })
                                        .name
                                        .clone();
                                    let charm =
                                        appstate.data.charms[result.charms_index].to_string();
                                    // let decorations_data = appstate.data.decorations.read().unwrap();
                                    let set = [head, body, arms, waist, legs, charm]
                                        .into_iter()
                                        .fold(Column::new(), |col, part| {
                                            let mut row = row![text(part.clone())
                                                .shaping(text::Shaping::Advanced)
                                                .width(Length::Fill)];
                                            if &part != "torso up" && !part.contains(",") {
                                                row = row.push(
                                                    button(
                                                        text("🗑️").shaping(text::Shaping::Advanced),
                                                    )
                                                    .on_press(Message::PartExcludedAdded(
                                                        part.clone(),
                                                    ))
                                                    .style(button::danger),
                                                );
                                            }
                                            col.push(row)
                                        });
                                    (
                                        torso_up_count,
                                        container(
                                            result
                                                .decorations_count_indices
                                                .iter()
                                                .filter(|(count, _)| *count > 0)
                                                .fold(set, |col, (count, index)| {
                                                    col.push(row![
                                                        text(format!(
                                                            "{count} x {}",
                                                            appstate.data.decorations[*index].name
                                                        ))
                                                        .width(Length::Fill),
                                                        button(
                                                            text("🗑️")
                                                                .shaping(text::Shaping::Advanced)
                                                        )
                                                        .on_press(Message::PartExcludedAdded(
                                                            appstate.data.decorations[*index]
                                                                .name
                                                                .clone()
                                                        ))
                                                        .style(button::danger)
                                                    ])
                                                }),
                                        )
                                        .style(container::bordered_box)
                                        .into(),
                                    )
                                })
                                .sorted_by_key(|&(x, _)| x)
                                .map(|(_, x)| x)
                        )
                        .padding(Padding::ZERO.right(SCROLLBAR_WIDTH))
                        .spacing(10)
                    )
                ],
                SearchStatus::Searching(_) =>
                    column![text("Searching...")].align_x(iced::Alignment::Center),
            }
            .width(Length::Fixed(280f32)),
            UIState::Charms(ref state) => column![
                button(text("Add")).on_press_maybe(
                    (state.skills.first().is_some_and(|x| x.points != 0)
                        && state.skills.len() <= 2)
                        .then_some(Message::CustomAdd)
                ),
                text(""),
                row![
                    button(text("Abort"))
                        .on_press(Message::Abort)
                        .width(Length::Fill),
                    button(text("Save"))
                        .on_press(Message::CustomSave)
                        .width(Length::Fill),
                ],
                text(""),
                row![text("Charms"),],
                text_input("search", &appstate.settings.skill_filter)
                    .on_input(Message::SkillFilterChanged),
                scrollable(Column::with_children(
                    state
                        .charms
                        .iter()
                        .enumerate()
                        .filter(|&(_index, charm)| contains_fuzzy(
                            &charm.to_string(),
                            &appstate.settings.skill_filter
                        ))
                        .map(|(index, charm)| row![
                            text(charm.to_string()).width(Length::Fill),
                            button(text("🗑️").shaping(text::Shaping::Advanced))
                                .on_press(Message::CustomRemove(index))
                                .style(button::danger),
                        ]
                        .padding(Padding::ZERO.right(SCROLLBAR_WIDTH))
                        .into())
                )),
            ]
            .width(Length::Fixed(280f32)),
            UIState::Armor(ref state) => column![
                button(text("Add"))
                    .on_press_maybe((!state.skill.is_empty()).then_some(Message::CustomAdd)),
                text(""),
                row![
                    button(text("Abort"))
                        .on_press(Message::Abort)
                        .width(Length::Fill),
                    button(text("Save"))
                        .on_press(Message::CustomSave)
                        .width(Length::Fill),
                ],
                text(""),
                text("Pieces"),
                text_input("search", &appstate.settings.skill_filter)
                    .on_input(Message::SkillFilterChanged),
                scrollable(Column::with_children(
                    state
                        .pieces
                        .iter()
                        .enumerate()
                        .filter(|&(_index, piece)| contains_fuzzy(
                            &ArmorState::piece_to_string(piece),
                            &appstate.settings.skill_filter
                        ))
                        .map(|(index, piece)| row![
                            text(ArmorState::piece_to_string(piece))
                                .width(Length::Fill)
                                .shaping(text::Shaping::Advanced),
                            button(text("🗑️").shaping(text::Shaping::Advanced))
                                .on_press(Message::CustomRemove(index))
                                .style(button::danger),
                        ]
                        .padding(Padding::ZERO.right(SCROLLBAR_WIDTH))
                        .into())
                ))
            ]
            .width(Length::Fixed(280f32)),
        }
    ]
    .spacing(10)
    .into()
}
