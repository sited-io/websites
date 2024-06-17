CREATE TABLE pages (
  page_id SERIAL PRIMARY KEY,
  website_id VARCHAR NOT NULL REFERENCES websites(website_id),
  user_id VARCHAR NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW() ON UPDATE NOW(),
  page_type VARCHAR NOT NULL,
  content_id VARCHAR NOT NULL,
  title VARCHAR NOT NULL,
  path VARCHAR NOT NULL,

  UNIQUE INDEX uq_pages_website_id_title (website_id, title),
  UNIQUE INDEX uq_pages_website_id_path (website_id, path)
)