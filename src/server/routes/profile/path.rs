#[derive(Debug)]
pub enum Path {
    Main,
    CoinHeld,
    Replies,
    Notifications,
    CoinCreated,
    Followers,
    Followings,
}

impl Path {
    pub fn as_str(&self) -> &'static str {
        match self {
            Path::Main => "/profile/:nickname",
            Path::CoinHeld => "/profile/:nickname/coins-held",
            Path::Replies => "/profile/:nickname/replies",
            Path::Notifications => "/profile/:nickname/notifications",
            Path::CoinCreated => "/profile/:nickname/coins-created",
            Path::Followers => "/profile/:nickname/followers",
            Path::Followings => "/profile/:nickname/followings",
        }
    }
}
