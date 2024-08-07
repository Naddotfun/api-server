#[derive(Debug)]
pub enum ProfilePath {
    Profile,
    CoinHeld,
    Replies,
    CoinCreated,
    Followers,
    Following,
}

impl ProfilePath {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Profile => "/profile/:user",
            Self::CoinHeld => "/profile/coins-held/:user",
            Self::Replies => "/profile/replies/:user",
            Self::CoinCreated => "/profile/coins-created/:user",
            Self::Followers => "/profile/followers/:user",
            Self::Following => "/profile/following/:user",
        }
    }

    pub fn docs_str(&self) -> &'static str {
        match self {
            Self::Profile => "/profile/{user}",
            Self::CoinHeld => "/profile/coins-held/{user}",
            Self::Replies => "/profile/replies/{user}",
            Self::CoinCreated => "/profile/coins-created/{user}",
            Self::Followers => "/profile/followers/{user}",
            Self::Following => "/profile/following/{user}",
        }
    }
}
