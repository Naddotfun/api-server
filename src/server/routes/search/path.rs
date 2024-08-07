pub enum Path {
    Search,
}

impl Path {
    pub fn as_str(&self) -> &'static str {
        match self {
            Path::Search => "/search/:token",
        }
    }
    pub fn docs_str(&self) -> &'static str {
        match self {
            Path::Search => "/search/{token}",
        }
    }
}
