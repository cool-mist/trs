# Terminal RSs


Keep the goal simple.

- just list the unread articles
- Have a way to mark an article as read/unread
- https://www.rssboard.org/rss-specification

## Database

Channels = Id | FeedName | Link | Atom/RSS | checksum

Articles = Id | ChannelId | Title | Link (unique) | Description | Published | Read

https://sqlite.org/lang_upsert.html

INSERT INTO Articles(ChannelId, Title, Link, Description, Published, 0)
  VALUES('Alice','704-555-1212','2018-05-08')
  ON CONFLICT(name) DO UPDATE SET
    Read=0,
    Published=excluded.Published
  WHERE excluded.Published>Articles.Published;

