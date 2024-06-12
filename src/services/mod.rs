mod website;

use tonic::Status;

use crate::api::sited_io::pagination::v1::{
    PaginationRequest, PaginationResponse,
};

pub use website::WebsiteService;

/// Returns limit and offset from PaginationRequest
fn get_limit_offset_from_pagination(
    request: Option<PaginationRequest>,
) -> Result<(u64, u64, PaginationResponse), Status> {
    let mut limit = 10;
    let mut offset = 0;
    let mut pagination = PaginationResponse {
        page: 1,
        size: limit,
        total_elements: 0,
    };

    if let Some(request) = request {
        if request.page < 1 {
            return Err(Status::invalid_argument(
                "pagination.page less than 1",
            ));
        }
        limit = request.size;
        offset = (request.page - 1) * request.size;
        pagination.page = request.page;
        pagination.size = request.size;
    }

    Ok((limit.into(), offset.into(), pagination))
}
