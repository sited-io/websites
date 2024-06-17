use deadpool_postgres::Pool;
use jwtk::jwk::RemoteJwksVerifier;
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
use crate::model::{Page, PageAsRel, Website};

use super::get_limit_offset_from_pagination;

pub struct PageService {
    pool: Pool,
    verifier: RemoteJwksVerifier,
}

impl PageService {
    const HOME_PAGE_PATH: &'static str = "/";

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
            path: page.path,
        }
    }

    fn page_type_from_request(page_type: i32) -> Result<PageType, Status> {
        let page_type = PageType::try_from(page_type).map_err(|_| {
            Status::invalid_argument(format!("Unknown page type {}", page_type))
        })?;
        if page_type == PageType::Uspecified {
            Err(Status::invalid_argument("Please provide known page type"))
        } else {
            Ok(page_type)
        }
    }

    fn get_path(is_home_page: bool, title: &String) -> String {
        if is_home_page {
            Self::HOME_PAGE_PATH.to_string()
        } else {
            slugify(title)
        }
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
        } = request.into_inner();
        let page_type = Self::page_type_from_request(page_type)?;
        let path = Self::get_path(is_home_page, &title);

        Website::get_for_user(&self.pool, &website_id, &user_id)
            .await?
            .ok_or_else(|| {
                Status::not_found(format!(
                    "Could not find website '{}'",
                    website_id
                ))
            })?;

        let created_page = Page::create(
            &self.pool,
            &website_id,
            &user_id,
            page_type.as_str_name(),
            &content_id,
            &title,
            &path,
        )
        .await?;

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
        } = request.into_inner();

        let page_type = match page_type {
            Some(p) => Some(Self::page_type_from_request(p)?.as_str_name()),
            None => None,
        };

        let path = match (is_home_page, &title) {
            (Some(is_home_page), Some(title)) => Some(Self::get_path(is_home_page, title)),
            (None, None) => None,
            _ => return Err(Status::invalid_argument("Please provide either both of title and is_home_page or none of them"))
        };

        let updated_page = Page::update(
            &self.pool, page_id, &user_id, page_type, content_id, title, path,
        )
        .await?;

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

        Page::delete(&self.pool, page_id, &user_id).await?;

        Ok(Response::new(DeletePageResponse {}))
    }
}
