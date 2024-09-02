#[derive(Debug)]
pub enum ProfilePath {
    Profile,
    TokenHeld,
    Replies,
    TokenCreated,
    Followers,
    Following,
}

impl ProfilePath {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Profile => "/profile/:user",
            Self::TokenHeld => "/profile/tokens-held/:user",
            Self::Replies => "/profile/replies/:user",
            Self::TokenCreated => "/profile/tokens-created/:user",
            Self::Followers => "/profile/followers/:user",
            Self::Following => "/profile/following/:user",
        }
    }

    pub fn docs_str(&self) -> &'static str {
        match self {
            Self::Profile => "/profile/{user}",
            Self::TokenHeld => "/profile/tokens-held/{user}",
            Self::Replies => "/profile/replies/{user}",
            Self::TokenCreated => "/profile/tokens-created/{user}",
            Self::Followers => "/profile/followers/{user}",
            Self::Following => "/profile/following/{user}",
        }
    }
}
