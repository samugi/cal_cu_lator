#[derive(Debug, serde::Deserialize)]
pub struct Item {
    pub name: String,
    pub values: Vec<f64>,
}
