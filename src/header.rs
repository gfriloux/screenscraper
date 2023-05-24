use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Header {
   #[serde(rename = "APIversion")]
   pub apiversion:       String,

   #[serde(rename = "dateTime")]
   pub datetime:         String,

   #[serde(rename = "commandRequested")]
   pub commandrequested: String,
   pub success:          String,
   pub error:            String
}
