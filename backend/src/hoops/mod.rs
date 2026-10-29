use salvo::prelude::*;

mod cors;
pub mod custom_middleware_example;
pub mod jwt;
pub use cors::cors_hoop;

#[handler]
pub async fn error_404(&self, res: &mut Response, ctrl: &mut FlowCtrl) {
    if let Some(StatusCode::NOT_FOUND) = res.status_code {
        res.render(Text::Html("404 Not Found"));
        ctrl.skip_rest();
    }
}
