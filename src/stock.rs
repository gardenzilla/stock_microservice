use chrono::prelude::*;
use packman::VecPackMember;
use serde::{Deserialize, Serialize};

// Price entity related to a SKU
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Stock {
  pub id: u32,
  pub name: String,
  pub description: String,
  pub created_at: DateTime<Utc>,
  pub created_by: u32,
}

impl Default for Stock {
  fn default() -> Self {
    Self {
      id: 0,
      name: String::default(),
      description: String::default(),
      created_at: Utc::now(),
      created_by: 0,
    }
  }
}

impl Stock
where
  Self: Sized,
{
  pub fn new(id: u32, name: String, description: String, created_by: u32) -> Self {
    Self {
      id,
      name,
      description,
      created_at: Utc::now(),
      created_by,
    }
  }
  pub fn update(&mut self, name: String, description: String) -> &Self {
    self.name = name;
    self.description = description;
    self
  }
}

impl VecPackMember for Stock {
  type Out = u32;

  fn get_id(&self) -> &Self::Out {
    &self.id
  }
}
