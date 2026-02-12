use salvo::prelude::*;

/// Middleware that executes logic after the next handler (Post-processing)
#[allow(dead_code)]
#[handler]
async fn post_processing_middleware(&self, req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
    ctrl.call_next(req, depot, res).await;
    // Middleware that executes logic after the next handler (Post-processing)
}

/// Middleware implementing the onion model with pre and post logic
#[allow(dead_code)]
#[handler]
async fn onion_model_middleware(&self, req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
    // Middleware that executes logic before the matched handler (Pre-processing)
    ctrl.call_next(req, depot, res).await;
    // Middleware that executes logic after the next handler (Post-processing)
}

/// Middleware that executes logic before the matched handler (Pre-processing)
#[allow(dead_code)]
#[handler]
async fn pre_processing_middleware(&self, req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
    //  Middleware that executes logic before the matched handler (Pre-processing)
    ctrl.call_next(req, depot, res).await;
}


/// Middleware to skip remaining handlers
#[allow(dead_code)]
#[handler]
async fn skip_handler_middleware(&self, _req: &mut Request, _depot: &mut Depot, _res: &mut Response, ctrl: &mut FlowCtrl) {
    // Middleware to skip remaining handlers
    ctrl.skip_rest();
}
