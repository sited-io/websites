CREATE TABLE pages (
  page_id SERIAL PRIMARY KEY,
  website_id VARCHAR NOT NULL REFERENCES websites(website_id),
  user_id VARCHAR NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  page_type VARCHAR NOT NULL,
  content_id VARCHAR NOT NULL,
  title VARCHAR NOT NULL,
  path VARCHAR NOT NULL
);

CREATE UNIQUE INDEX uq_pages_website_id_title
  ON pages(website_id, title);

CREATE UNIQUE INDEX uq_pages_website_id_path
  ON pages(website_id, path);

CREATE TRIGGER update_shops_updated_at BEFORE UPDATE
    ON pages FOR EACH ROW EXECUTE PROCEDURE 
    updated_at_now();
