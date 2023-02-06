use std::error::Error;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use chrono::{Local, NaiveDateTime};
use rnotifylib::message::{Level, Message};
use rnotifylib::message::builder::MessageBuilder;
use rnotifylib::message::component::Component;
use rnotifylib::message::detail_builder::FormattedStringAppendable;
use serde::{Deserialize, Serialize};
use crate::resource::anticipation::IndustryNotificationType::*;

#[derive(Debug, Default, Clone)]
pub struct IndustryNotificationResource {
    last_checked: NaiveDateTime,
    data: Option<DfsIndustryNotification>,
}

impl IndustryNotificationResource {
    pub fn get_last_checked(&self) -> &NaiveDateTime {
        &self.last_checked
    }

    pub fn get_last_data(&self) -> &Option<DfsIndustryNotification> {
        &self.data
    }

    pub fn update(&mut self, new: DfsIndustryNotification) {
        self.data = Some(new);
        self.last_checked = Local::now().naive_local();
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IndustryNotificationType {
    RequirementPublished,
    RequirementAnticipated,
    RequirementCancelled,
}

impl FromStr for IndustryNotificationType {
    type Err = UnknownIndustryNotificationType;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "Requirement Published" => Ok(RequirementPublished),
            "Anticipated Requirement Notice" => Ok(RequirementAnticipated),
            "Test DFS Requirement not issued" => Ok(RequirementCancelled),
            "Live DFS Requirement not issued" => Ok(RequirementCancelled),
            "Requirement Cancelled" => Ok(RequirementCancelled),
            _ => Err(UnknownIndustryNotificationType { got: s.to_owned() })
        }
    }
}

#[derive(Debug)]
pub struct UnknownIndustryNotificationType {
    got: String,
}

impl Display for UnknownIndustryNotificationType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown notification type '{}'", self.got)
    }
}

impl Error for UnknownIndustryNotificationType {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DfsIndustryNotification {
    anticipation_type: IndustryNotificationType,
    when: NaiveDateTime,
    description: String,
}

impl DfsIndustryNotification {
    pub fn new(anticipation_type: IndustryNotificationType,
               when: NaiveDateTime,
               description: String) -> Self {
        Self {
            anticipation_type,
            when,
            description,
        }
    }

    pub fn get_type(&self) -> &IndustryNotificationType {
        &self.anticipation_type
    }

    pub fn get_when(&self) -> &NaiveDateTime {
        &self.when
    }

    pub fn create_message(&self, live_or_test: &str) -> Message {
        let mut message_builder = MessageBuilder::new();

        message_builder
            .level(Level::Info)
            .title(format!("Dfs Industry Notification - {}", live_or_test))
            .body(|body| {
                body.raw(format!("{:?} - {}", self, live_or_test));

                body.text_block(|block| {
                    block.append_plain(format!("{:?} at {}", self.anticipation_type, self.when));
                });

                body.section("Description", |builder| {
                    builder.append_plain(&self.description);
                });
            })
            .component(Component::from("dfs/industry_notification"))
            .author("dfs_alert");

        message_builder.build()
    }
}