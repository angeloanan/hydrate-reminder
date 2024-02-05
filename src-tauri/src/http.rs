use std::sync::LazyLock;

pub const REQWEST_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; hydrate_reminder/0.1; +https://github.com/angeloanan/hydrate-reminder)")
        .build()
        .expect("Unable to create reqwest client!")
});
