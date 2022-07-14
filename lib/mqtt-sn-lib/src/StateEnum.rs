use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;


#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, EnumIter, Copy, Clone)]
pub enum StateEnum {
    ACTIVE,
    DISCONNECTED,
    ASLEEP,
    AWAKE,
    LOST,
}
