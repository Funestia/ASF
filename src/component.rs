use serde::Deserialize;

#[derive(Default, Clone, Deserialize, Debug)]
#[serde(default)]
pub struct Component {
    pub name: String,
    special: Option<String>,
}

impl Component {
    pub fn japanese(&self) -> bool {
        self.special == Some("jEvent".to_owned())
    }
    pub fn event(&self) -> bool {
        self.special == Some("Event".to_owned()) || self.japanese()
    }

}
