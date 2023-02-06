use serde::{Deserialize, Serialize};
use crate::resource::anticipation::DfsIndustryNotification;

#[derive(Serialize, Deserialize, Debug)]
pub struct AllSavedData {
    pub test: SavedData,
    pub live: SavedData,
}

impl AllSavedData {
    pub fn get_test(&self) -> &SavedData {
        &self.test
    }

    pub fn get_live(&self) -> &SavedData {
        &self.live
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SavedData {
    pub industry_notification: Option<DfsIndustryNotification>,
}

impl SavedData {
    pub fn get_industry_notification(&self) -> &Option<DfsIndustryNotification> {
        &self.industry_notification
    }
}