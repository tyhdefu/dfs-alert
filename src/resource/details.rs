use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

pub struct CheckedDfsSupplierResource {
    last_checked: NaiveDateTime,
    data: BidData,
}

impl CheckedDfsSupplierResource {
    pub fn get_last_checked(&self) -> &NaiveDateTime {
        &self.last_checked
    }
}

pub struct BidData {
    bids: Vec<Bid>
}

pub struct Bid {
    date: NaiveDate,
    provider: String,
    from: NaiveTime,
    to: NaiveTime,
}