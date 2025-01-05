-- Add migration script here
CREATE TABLE IF NOT EXISTS `subscribers` (
  `chat_id` INTEGER NOT NULL,
  PRIMARY KEY (`chat_id`)
);
