use dotenv::dotenv;
use mongodb::{
    options::{ClientOptions, ResolverConfig},
    Client, Database,
};
use std::env;

pub struct Tweetbook {
    pub db: Database,
}

impl Tweetbook {
    pub async fn init() -> Self {
        dotenv().ok();

        let uri = env::var("MONGO_URI").unwrap();

        let options = ClientOptions::parse_with_resolver_config(&uri, ResolverConfig::cloudflare())
            .await
            .unwrap();

        let client = Client::with_options(options).unwrap();
        let db = client.database("TweetBook");
        Tweetbook { db }
    }
}
