use rss::Channel;
use chrono::{DateTime, Utc};
use rusqlite::Result;
use html2md::parse_html;
use super::db::{Database, Item};

pub fn update(channel_url: &str, db: &Database) -> Result<()> {
    let channel = Channel::from_url(channel_url).unwrap();
    // let title = channel.title();
    let now = Utc::now().timestamp();
    for it in channel.items() {
        let item = Item {
            read: false,
            channel: channel_url.to_string(),
            title: it.title().map(Into::into),
            url: it.link().map(Into::into),
            retrieved_at: now,
            published_at: match it.pub_date().map(Into::into) {
                Some(pub_date) => {
                    let dt = DateTime::parse_from_rfc2822(pub_date).unwrap();
                    Some(dt.timestamp())
                },
                None => None
            },
            description: match it.description().map(Into::into) {
                Some(desc) => Some(parse_html(desc)),
                None => None
            },
        };
        db.add_item(&item)?
    }
    Ok(())
}
