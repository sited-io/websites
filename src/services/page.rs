use deadpool_postgres::Pool;
use jwtk::jwk::RemoteJwksVerifier;
use serde_json::Value;
use slug::slugify;
use tonic::{async_trait, Request, Response, Status};

use crate::api::sited_io::websites::v1::page_service_server::{
    self, PageServiceServer,
};
use crate::api::sited_io::websites::v1::{
    CreatePageRequest, CreatePageResponse, DeletePageRequest,
    DeletePageResponse, GetPageRequest, GetPageResponse, ListPagesRequest,
    ListPagesResponse, PageResponse, PageType, UpdatePageRequest,
    UpdatePageResponse,
};
use crate::auth::get_user_id;
use crate::i64_to_u32;
use crate::model::{Page, PageAsRel, StaticPage, Website};

use super::get_limit_offset_from_pagination;

pub struct PageService {
    pool: Pool,
    verifier: RemoteJwksVerifier,
}

impl PageService {
    pub const HOME_PAGE_PATH: &'static str = "/";
    pub const DEFAULT_HOME_PAGE_TITLE: &'static str = "Home";

    pub fn build(
        pool: Pool,
        verifier: RemoteJwksVerifier,
    ) -> PageServiceServer<Self> {
        PageServiceServer::new(Self { pool, verifier })
    }

    pub fn to_response(page: impl Into<PageAsRel>) -> PageResponse {
        let page: PageAsRel = page.into();
        PageResponse {
            page_id: page.page_id,
            page_type: PageType::from_str_name(&page.page_type).unwrap().into(),
            content_id: page.content_id,
            title: page.title,
            is_home_page: page.is_home_page,
            path: page.path,
        }
    }

    fn page_type_from_request(page_type: i32) -> Result<PageType, Status> {
        let page_type = PageType::try_from(page_type).map_err(|_| {
            Status::invalid_argument(format!("Unknown page type {}", page_type))
        })?;
        if page_type == PageType::Unspecified {
            Err(Status::invalid_argument("Please provide known page type"))
        } else {
            Ok(page_type)
        }
    }

    fn get_slugified_path(title: &String) -> String {
        format!("/{}", slugify(title))
    }

    async fn make_current_home_page_not_home_page(
        &self,
        website_id: &String,
        user_id: &String,
    ) -> Result<(), Status> {
        if let Some(current_home_page) =
            Page::get_home_page(&self.pool, website_id).await?
        {
            Page::update(
                &self.pool,
                current_home_page.page_id,
                user_id,
                None,
                None,
                None,
                Some(false),
                Some(Self::get_slugified_path(&current_home_page.title)),
            )
            .await?;
        }

        Ok(())
    }

    async fn ensure_static_page(
        &self,
        page_id: i64,
        website_id: &String,
        user_id: &String,
    ) -> Result<(), Status> {
        if StaticPage::get(&self.pool, page_id).await?.is_none() {
            StaticPage::create(
                &self.pool,
                page_id,
                website_id,
                user_id,
                Value::Array(Vec::new()),
            )
            .await?;
        }

        Ok(())
    }
}

#[async_trait]
impl page_service_server::PageService for PageService {
    async fn create_page(
        &self,
        request: Request<CreatePageRequest>,
    ) -> Result<Response<CreatePageResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let CreatePageRequest {
            website_id,
            page_type,
            content_id,
            title,
            is_home_page,
            path,
        } = request.into_inner();

        let page_type = Self::page_type_from_request(page_type)?;

        let mut path = path.unwrap_or_else(|| Self::get_slugified_path(&title));

        Website::get_for_user(&self.pool, &website_id, &user_id)
            .await?
            .ok_or_else(|| {
                Status::not_found(format!(
                    "Could not find website '{}'",
                    website_id
                ))
            })?;

        if is_home_page {
            self.make_current_home_page_not_home_page(&website_id, &user_id)
                .await?;

            path = Self::HOME_PAGE_PATH.to_string();
        }

        let created_page = Page::create(
            &self.pool,
            &website_id,
            &user_id,
            page_type.as_str_name(),
            &content_id,
            &title,
            is_home_page,
            &path,
        )
        .await?;

        if page_type == PageType::Static {
            self.ensure_static_page(
                created_page.page_id,
                &website_id,
                &user_id,
            )
            .await?;
        }

        Ok(Response::new(CreatePageResponse {
            page: Some(Self::to_response(created_page)),
        }))
    }

    async fn get_page(
        &self,
        request: Request<GetPageRequest>,
    ) -> Result<Response<GetPageResponse>, Status> {
        let GetPageRequest {
            page_id,
            website_id,
            path,
        } = request.into_inner();

        let found_page = match (page_id, website_id, path) {
            (Some(page_id), _, _) => Page::get(&self.pool, page_id).await?,
            (_, Some(website_id), Some(path)) => {
                Page::get_by_path(&self.pool, &website_id, &path).await?
            }
            _ => return Err(Status::invalid_argument(
                "Please provide either page_id or both of website_id and path",
            )),
        };

        Ok(Response::new(GetPageResponse {
            page: found_page.map(Self::to_response),
        }))
    }

    async fn list_pages(
        &self,
        request: Request<ListPagesRequest>,
    ) -> Result<Response<ListPagesResponse>, Status> {
        let ListPagesRequest {
            website_id,
            pagination,
        } = request.into_inner();

        let (limit, offset, mut pagination) =
            get_limit_offset_from_pagination(pagination)?;

        let (found_pages, count) =
            Page::list(&self.pool, website_id, limit, offset).await?;

        pagination.total_elements = i64_to_u32(count)?;

        Ok(Response::new(ListPagesResponse {
            pages: found_pages.into_iter().map(Self::to_response).collect(),
            pagination: Some(pagination),
        }))
    }

    async fn update_page(
        &self,
        request: Request<UpdatePageRequest>,
    ) -> Result<Response<UpdatePageResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let UpdatePageRequest {
            page_id,
            page_type,
            content_id,
            title,
            is_home_page,
            mut path,
        } = request.into_inner();

        if matches!(is_home_page, Some(true)) {
            let found_page =
                Page::get(&self.pool, page_id).await?.ok_or_else(|| {
                    Status::not_found("Could not find page to update")
                })?;

            self.make_current_home_page_not_home_page(
                &found_page.website_id,
                &user_id,
            )
            .await?;

            path = Some(Self::HOME_PAGE_PATH.to_string());
        }

        let page_type = match page_type {
            Some(p) => Some(Self::page_type_from_request(p)?.as_str_name()),
            None => None,
        };

        let updated_page = Page::update(
            &self.pool,
            page_id,
            &user_id,
            page_type,
            content_id,
            title,
            is_home_page,
            path,
        )
        .await?;

        if page_type.is_some_and(|p| p == PageType::Static.as_str_name()) {
            self.ensure_static_page(
                page_id,
                &updated_page.website_id,
                &user_id,
            )
            .await?;
        }

        Ok(Response::new(UpdatePageResponse {
            page: Some(Self::to_response(updated_page)),
        }))
    }

    async fn delete_page(
        &self,
        request: Request<DeletePageRequest>,
    ) -> Result<Response<DeletePageResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let DeletePageRequest { page_id } = request.into_inner();

        let found_page = Page::get(&self.pool, page_id)
            .await?
            .ok_or_else(|| Status::not_found(""))?;

        if found_page.path == Self::HOME_PAGE_PATH {
            return Err(Status::invalid_argument("Cannot delete home page"));
        }

        StaticPage::delete(&self.pool, page_id, &user_id).await?;

        Page::delete(&self.pool, page_id, &user_id).await?;

        Ok(Response::new(DeletePageResponse {}))
    }
}
