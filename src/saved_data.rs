use serde::{Deserialize, Serialize};
use crate::resource::anticipation::DfsIndustryNotification;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AllSavedData {
    #[serde(default)]
    pub test: SavedData,
    #[serde(default)]
    pub live: SavedData,
    #[serde(default)]
    pub new_2023: SavedData,
}

impl AllSavedData {
    pub fn get_test(&self) -> &SavedData {
        &self.test
    }

    pub fn get_live(&self) -> &SavedData {
        &self.live
    }

    pub fn get_new(&self) -> &SavedData {
        &self.new_2023
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SavedData {
    #[serde(default)]
    pub industry_notification: Option<DfsIndustryNotification>,
}

impl SavedData {
    pub fn get_industry_notification(&self) -> &Option<DfsIndustryNotification> {
        &self.industry_notification
    }
}