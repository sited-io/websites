CREATE TABLE customizations (
  website_id VARCHAR NOT NULL PRIMARY KEY REFERENCES websites(website_id),
  user_id VARCHAR NOT NULL,
  primary_color VARCHAR,
  secondary_color VARCHAR
);
