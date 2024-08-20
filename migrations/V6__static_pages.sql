CREATE TABLE static_pages (
  page_id SERIAL PRIMARY KEY REFERENCES pages(page_id),
  website_id VARCHAR NOT NULL REFERENCES websites(website_id),
  user_id VARCHAR NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW() ON UPDATE NOW(),
  components JSON
);
