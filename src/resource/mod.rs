use chrono::NaiveDateTime;
use error::ResourceNotFoundError;
use crate::resource::anticipation::{IndustryNotificationResource};
use serde::Deserialize;

pub mod error;
pub mod anticipation;
pub mod details;

#[derive(Deserialize, Debug)]
pub struct AvailableResources {
    resources: Vec<Resource>,
}

const PARTIAL_INDUSTRY_NOTIFICATION_MATCH: &str = "service_update_industry_notifications_";
const NEW_INDUSTRY_NOTIFICATION_MATCH: &str = "dfs_industry_notification";

impl AvailableResources {
    pub fn get_dfs_industry_notification_resource(&self) -> Result<Resource, ResourceNotFoundError> {
        for resource in &self.resources {
            if resource.get_name().starts_with(PARTIAL_INDUSTRY_NOTIFICATION_MATCH)
                || resource.get_name().starts_with(NEW_INDUSTRY_NOTIFICATION_MATCH) {
                return Ok(resource.clone())
            }
        }
        Err(ResourceNotFoundError::new(format!("{}*|{}*", PARTIAL_INDUSTRY_NOTIFICATION_MATCH, NEW_INDUSTRY_NOTIFICATION_MATCH)))
    }

    pub fn get_dfs_supplier_details_source(&self) -> Option<Resource> {
        todo!()
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct Resource {
    name: String,
    last_modified: NaiveDateTime,
    path: String,
}

impl Resource {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_last_modified(&self) -> &NaiveDateTime {
        &self.last_modified
    }

    pub fn get_path(&self) -> &str {
        &self.path
    }
}

pub struct PreviousResources {
    anticipated: IndustryNotificationResource,
    //details: CheckedDfsSupplierResource,
}

impl PreviousResources {
    pub fn create(anticipated: IndustryNotificationResource) -> Self  {
        Self {
            anticipated,
        }
    }

    pub fn get_anticipated(&mut self) -> &mut IndustryNotificationResource {
        &mut self.anticipated
    }

    /*pub fn get_supplier_details(&self) -> &CheckedDfsSupplierResource {
        &self.details
    }*/
}