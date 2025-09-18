use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum Language {
    German,
    #[default]
    English,
    Spanish,
    French,
    Italian,
    Japanese,
}
impl Language {
    pub fn all() -> Vec<Language> {
        vec![
            Self::German,
            Self::English,
            Self::Spanish,
            Self::French,
            Self::Italian,
            Self::Japanese,
        ]
    }
}
impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::German => "Deutsch",
                Self::French => "Français",
                Self::English => "English",
                Self::Japanese => "日本語",
                Self::Italian => "Italiano",
                Self::Spanish => "Español",
            }
        )
    }
}
#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum GatheringHallRank {
    #[default]
    HR1 = 1,
    HR2,
    HR3,
    HR4,
    HR5,
    HR6,
    HR7,
    HR8,
    G1,
    G2,
    G3,
    All,
}

impl std::fmt::Display for GatheringHallRank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GatheringHallRank::HR1 => "HR1",
                GatheringHallRank::HR2 => "HR2",
                GatheringHallRank::HR3 => "HR3",
                GatheringHallRank::HR4 => "HR4",
                GatheringHallRank::HR5 => "HR5",
                GatheringHallRank::HR6 => "HR6",
                GatheringHallRank::HR7 => "HR7",
                GatheringHallRank::HR8 => "HR8",
                GatheringHallRank::G1 => "G1",
                GatheringHallRank::G2 => "G2",
                GatheringHallRank::G3 => "G3",
                GatheringHallRank::All => "All",
            }
        )
    }
}

impl GatheringHallRank {
    pub fn all() -> Vec<GatheringHallRank> {
        vec![
            GatheringHallRank::HR1,
            GatheringHallRank::HR1,
            GatheringHallRank::HR2,
            GatheringHallRank::HR3,
            GatheringHallRank::HR4,
            GatheringHallRank::HR5,
            GatheringHallRank::HR6,
            GatheringHallRank::HR7,
            GatheringHallRank::HR8,
            GatheringHallRank::G1,
            GatheringHallRank::G2,
            GatheringHallRank::G3,
            GatheringHallRank::All,
        ]
    }
}
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum WeaponType {
    Both = 0,
    #[default]
    Melee = 1,
    Marksman = 2,
}
impl std::fmt::Display for WeaponType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                WeaponType::Both => "Both",
                WeaponType::Melee => "Melee",
                WeaponType::Marksman => "Marksman",
            }
        )
    }
}
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum ArmorType {
    #[default]
    Head,
    Arms,
    Chest,
    Waist,
    Legs
}
impl ArmorType {
    pub fn all() -> Vec<ArmorType> {
        vec![
            Self::Head,
            Self::Arms,
            Self::Chest,
            Self::Waist,
            Self::Legs
        ]
    }
}
impl std::fmt::Display for ArmorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ArmorType::Head => "Head",
                ArmorType::Arms => "Arms",
                ArmorType::Chest => "Chest",
                ArmorType::Waist => "Waist",
                ArmorType::Legs => "Legs"
            }
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum Sex {
    #[default]
    Male = 1,
    Female = 2,
}

impl std::fmt::Display for Sex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Sex::Male => "Male",
                Sex::Female => "Female",
            }
        )
    }
}
