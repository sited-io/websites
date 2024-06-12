use std::fmt::Debug;

use tower_http::{
    classify::GrpcFailureClass,
    trace::{OnFailure, OnRequest, OnResponse},
};

const HEALTH_PATH: &str = "/grpc.health.v1.Health/Check";
const REFLECTION_PATH: &str =
    "/grpc.reflection.v1alpha.ServerReflection/ServerReflectionInfo";

#[derive(Debug, Clone, Default)]
pub struct LogOnRequest {}

impl<B> OnRequest<B> for LogOnRequest {
    fn on_request(
        &mut self,
        request: &tonic::codegen::http::Request<B>,
        _span: &tracing::Span,
    ) {
        if request.uri().path() == HEALTH_PATH
            || request.uri().path() == REFLECTION_PATH
        {
            return;
        }

        tracing::log::debug!(
            target: "grpc-request",
            "{:?} {} {} {:?}",
            request.version(),
            request.method(),
            request.uri(),
            request.headers()
        );
    }
}

#[derive(Debug, Clone, Default)]
pub struct LogOnResponse {}

impl<B> OnResponse<B> for LogOnResponse {
    fn on_response(
        self,
        response: &tonic::codegen::http::Response<B>,
        _latency: std::time::Duration,
        _span: &tracing::Span,
    ) {
        if response.status().is_success() {
            return;
        }

        tracing::log::debug!(
            target: "grpc-response",
            "{:?} {} {:?}",
            response.version(),
            response.status(),
            response.headers(),
        );
    }
}

#[derive(Debug, Clone, Default)]
pub struct LogOnFailure {}

impl OnFailure<GrpcFailureClass> for LogOnFailure {
    fn on_failure(
        &mut self,
        failure_classification: GrpcFailureClass,
        _latency: std::time::Duration,
        _span: &tracing::Span,
    ) {
        tracing::log::error!(
            target: "grpc-failure",
            "{:?}",
            failure_classification,
        );
    }
}
