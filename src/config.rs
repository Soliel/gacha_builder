use rocket::serde::{Serialize, Deserialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct GachaConfig {
    test1: String,
    testconf: String
}

impl Default for GachaConfig {
    fn default() -> Self {
        Self { 
            test1: String::from("This value is from default!"),
            testconf: String::from("This value is also from default!")
        }
    }
}