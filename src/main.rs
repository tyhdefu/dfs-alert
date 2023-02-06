use std::error::Error;
use std::time::Duration;
use chrono::{NaiveDate, NaiveTime};
use rnotifylib::config::Config;
use rnotifylib::message::builder::MessageBuilder;
use rnotifylib::message::detail_builder::FormattedStringAppendable;
use rnotifylib::message::Level;
use rnotifylib::message_router::MessageRouter;
use serde::{Deserialize, Deserializer};
use serde::de::{Error as SerdeError, Unexpected};
use crate::resource::{AvailableResources, PreviousResources};
use crate::resource::anticipation::{DfsIndustryNotification, IndustryNotificationResource, IndustryNotificationType};
use crate::resource::details::CheckedDfsSupplierResource;
use crate::saved_data::{AllSavedData, SavedData};

mod resource;
mod saved_data;

const OUR_SUPPLIER: &str = "OctopusEnergyLimited";
const OUR_REGION: &str = "East Midlands";
// https://data.nationalgrideso.com/dfs/demand-flexibility-service-live-events/r/dfs_utilisation_report_-_live

const LIVE_URL: &str = "https://data.nationalgrideso.com/dfs/demand-flexibility-service-live-events/datapackage.json";
const TEST_URL: &str = "https://data.nationalgrideso.com/dfs/demand-flexibility-service-test-events/datapackage.json";

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let config_str = std::fs::read_to_string("routing.toml")
        .expect("Failed to read config file");
    let routing_config: Config = toml::from_str(&config_str)
        .expect("Config file format invalid.");

    let message_router = MessageRouter::from_config(routing_config);

    let i_notification_resource = IndustryNotificationResource::default();

    let mut live_resources = PreviousResources::create(i_notification_resource.clone());
    let mut test_resources = PreviousResources::create(i_notification_resource);

    let state = load_state();
    match state {
        Ok(saved_data) => {
            println!("Loaded previous state: {:?}", saved_data);
            if let Some(old) = saved_data.get_live().get_industry_notification() {
                live_resources.get_anticipated().update(old.clone())
            }
            if let Some(old) = saved_data.get_test().get_industry_notification() {
                test_resources.get_anticipated().update(old.clone());
            }
        }
        Err(err) => {
            eprintln!("Failed to load data: {}", err);
        }
    }

    loop {
        let mut changed = false;
        changed |= run("live", &mut live_resources, LIVE_URL, &message_router).await;
        changed |= run("test", &mut test_resources, TEST_URL, &message_router).await;

        if changed {
            let to_save = AllSavedData {
                test: SavedData {
                    industry_notification: test_resources.get_anticipated().get_last_data().clone(),
                },
                live: SavedData {
                    industry_notification: live_resources.get_anticipated().get_last_data().clone(),
                },
            };
            println!("State changed, saving {:?}", to_save);
            match save_state(&to_save) {
                Ok(_) => {
                    println!("Successfully saved state");
                }
                Err(err) => {
                    eprintln!("Failed to save state: {:?}", err);
                }
            }
        }

        tokio::time::sleep(Duration::from_millis(30*20*1000)).await;
    }
}

const STATE_FILE: &str = "state.json";

fn load_state() -> Result<AllSavedData, Box<dyn Error>> {
    let s = std::fs::read_to_string(STATE_FILE)?;
    Ok(serde_json::from_str(&s)?)
}

fn save_state(data: &AllSavedData) -> Result<(), Box<dyn Error>> {
    let s = serde_json::to_string(data)?;
    Ok(std::fs::write(STATE_FILE, s)?)
}

async fn run(name: &str, resources: &mut PreviousResources, url: &str, router: &MessageRouter) -> bool {
    println!("Running '{}'", name);
    match check_for_changes(resources, url).await {
        Ok(None) => {
            println!("Nothing changed on {} resource", name);
        },
        Ok(Some(change)) => {
            let message = match change {
                NewPossibleEvent::Expected(notification) => notification.create_message(name),
                NewPossibleEvent::Confirmed(notification) => notification.create_message(name),
                NewPossibleEvent::Cancelled(notification) => notification.create_message(name),
                NewPossibleEvent::OurSupplierConfirmed => {
                    todo!()
                }
            };
            match router.route(&message) {
                Ok(amt) => {
                    println!("Informed {} destinations", amt);
                }
                Err(send_errors) => {
                    eprintln!("Errors informing some destinations:");
                    eprintln!("{}", send_errors);
                }
            };
            return true;
        }
        Err(err) => {
            let err_msg = format!("Error checking for changes on {} resources, {:?}", name, err);
            eprintln!("{}", err_msg);
            let mut builder = MessageBuilder::new();
            builder
                .title("Error checking demand flexibility service")
                .level(Level::Error)
                .component("dfs_alert/error".into())
                .author("dfs_alert")
                .body(|body| {
                    body.raw(err_msg.clone());
                    body.text_block(|text| {
                        text.append_plain(err_msg);
                    });
                });
            let message = builder.build();
            match router.route(&message) {
                Ok(count) => {
                    println!("Informed {} destinations of error.", count);
                }
                Err(err) => {
                    eprintln!("Failed to notify destinations of error checking DFS:");
                    eprintln!("{}", err)
                }
            }
        }
    }
    false
}

async fn check_for_changes(previous: &mut PreviousResources, url: &str) -> Result<Option<NewPossibleEvent>, Box<dyn Error>> {
    let available_resources: AvailableResources = reqwest::get(url).await?.json().await?;
    let anticipated = available_resources.get_dfs_industry_notification_resource()?;

    //let supplier_details = available_resources.get_dfs_supplier_details_source()?;

    let mut anticipated_event = None;

    if anticipated.get_last_modified() > previous.get_anticipated().get_last_checked() {
        // Anticipated changed.
        let new_data = read_industry_notification_data(anticipated.get_path()).await?;

        let previous_data = previous.get_anticipated();

        if previous_data.get_last_data()
            .as_ref()
            .filter(|old| old == &&new_data)
            .is_none() {
            // Data has actually changed.
            let new_event = match new_data.get_type() {
                IndustryNotificationType::RequirementAnticipated => NewPossibleEvent::Expected(new_data.clone()),
                IndustryNotificationType::RequirementPublished => NewPossibleEvent::Confirmed(new_data.clone()),
                IndustryNotificationType::RequirementCancelled => NewPossibleEvent::Cancelled(new_data.clone()),
            };
            anticipated_event = Some(new_event);
            previous_data.update(new_data);
        }
    }

    if anticipated_event.is_some() {
        return Ok(anticipated_event);
    }

    //if supplier_details.get_last_modified() > previous.get_supplier_details().get_last_checked() {
    //    let parsed = read_supplier_details(anticipated.get_path());
    //}

    Ok(None)
}

async fn read_industry_notification_data(url: &str) -> Result<DfsIndustryNotification, Box<dyn Error>> {
    let string = reqwest::get(url).await?.text().await?;
    let string = string.trim();
    //println!("{}", string);
    let mut reader = csv::Reader::from_reader(string.as_bytes());
    let mut parsed = vec![];
    for record in reader.deserialize() {
        let record: DfsIndustryNotificationResponse = record?;
        //println!("{:?}", record);
        parsed.push(record);
    }
    if parsed.is_empty() {
        panic!("No records returned");
    }
    let first = parsed.remove(0);
    Ok(first.into_notification()?)
}

fn read_supplier_details(url: &str) -> CheckedDfsSupplierResource {
    todo!()
}


#[derive(Debug)]
enum NewPossibleEvent {
    Expected(DfsIndustryNotification),
    Confirmed(DfsIndustryNotification),
    Cancelled(DfsIndustryNotification),
    OurSupplierConfirmed,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DfsIndustryNotificationResponse {
    date: NaiveDate,
    #[serde(rename = "Status")]
    description: String,
    #[serde(rename = "Type")]
    notification_type: String,
    #[serde(deserialize_with = "naive_date_time_from_str")]
    time: NaiveTime,
}

fn naive_date_time_from_str<'de, D>(deserializer: D) -> Result<NaiveTime, D::Error>
    where D: Deserializer<'de> {
    let s = String::deserialize(deserializer)?;
    Ok(NaiveTime::parse_from_str(&s, "%H:%M")
        .map_err(|err| D::Error::invalid_value(Unexpected::Other(&s), &"HH:MM"))?)
}

impl DfsIndustryNotificationResponse {
    pub fn into_notification(self) -> Result<DfsIndustryNotification, Box<dyn Error>> {
        Ok(DfsIndustryNotification::new(
            self.notification_type.parse()?,
            self.date.and_time(self.time),
            self.description
        ))
    }
}

#[cfg(test)]
mod test {
    use crate::DfsIndustryNotificationResponse;

    #[test]
    fn test_deserialize() {
        let s = "Date,Status,Type,Time
2023-01-23,DFS Service Requirement has been published for tomorrow. Please view the service requirement file in this dataset for details of the required volumes and times. This will be Energy Tagged ,Requirement Published,14:30
2023-01-23,This is an indication that a DFS Service Requirement might be published today at 14:30. This will be Energy Tagged. ,Anticipated Requirement Notice ,10:00
2023-01-22,DFS Service Requirement has been published for tomorrow. Please view the service requirement file in this dataset for details of the required volumes and times. This will be Energy Tagged ,Requirement Published,14:30
";
        let mut rdr = csv::Reader::from_reader(s.as_bytes());
        for record in rdr.deserialize() {
            let record: DfsIndustryNotificationResponse = record.expect("Should be able to deserialize");
            println!("record: {:?}", record);
            let parsed = record.into_notification().expect("Should be able to parse");
            println!("Parsed: {:?}", parsed);
        }
    }
}