//! simulate domain — dry-run predictions without mutating state
use crate::app::context::AppContext;
use crate::errors::CoreResult;

pub fn doctor(ctx: &AppContext) -> CoreResult<()> {
    crate::domains::doctor::simulate(ctx)
}

pub fn update(ctx: &AppContext) -> CoreResult<()> {
    crate::domains::update::simulate(ctx)
}
