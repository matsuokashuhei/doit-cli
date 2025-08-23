use chrono::NaiveDateTime;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DoItError {
    #[error("The 'to' date {} must be after 'from' date {}.", .to, .from)]
    FromAfterTo {
        from: NaiveDateTime,
        to: NaiveDateTime,
    },
}
