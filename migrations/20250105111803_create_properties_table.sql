-- Add migration script here
CREATE TABLE IF NOT EXISTS `properties` (
  -- Probably better to have a unique ID which isn't based on the website it was posted on.
  `url` varchar(1024) NOT NULL,
  PRIMARY KEY (`url`)
);
