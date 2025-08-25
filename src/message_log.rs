use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
  User,
  Model,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MsgType {
  Text,
  Image,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
  pub role: Role,
  #[serde(rename = "type")]
  pub kind: MsgType,
  pub content: String,
}

#[derive(Default, Debug)]
pub struct MessageLog {
  raw: Vec<Message>,
}

impl MessageLog {
  pub fn new() -> Self {
    Self { raw: Vec::new() }
  }

  pub fn add_user(&mut self, content: impl Into<String>) {
    self.raw.push(Message {
      role: Role::User,
      kind: MsgType::Text,
      content: content.into(),
    });
  }

  pub fn add_model(&mut self, content: impl Into<String>) {
    self.raw.push(Message {
      role: Role::Model,
      kind: MsgType::Text,
      content: content.into(),
    });
  }

  pub fn extend(&mut self, mut msgs: Vec<Message>) {
    self.raw.append(&mut msgs);
  }

  pub fn raw(&self) -> &Vec<Message> {
    &self.raw
  }
}
