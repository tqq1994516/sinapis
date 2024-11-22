use regex::Regex;

#[derive(Clone)]
pub struct OpenApiState {
    pub openapi: Vec<Regex>,
}
