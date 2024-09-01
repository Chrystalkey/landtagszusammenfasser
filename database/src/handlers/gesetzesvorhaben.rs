use std::sync::Arc;

use crate::infra::api::{CUPPayload, CUPResponse};
use crate::infra::db::connection as dbcon;
use crate::{error::LTZFError, infra::api as ifapi, AppState};
use axum::{extract::State, Json};
use diesel_interaction::*;
use uuid::Uuid;

async fn handle_gesvh(
    State(app): State<Arc<AppState>>,
    Json(cupdate): Json<ifapi::CUPUpdate>,
) -> std::result::Result<Json<CUPResponse>, LTZFError> {
    let gesvh = if let CUPPayload::GesVH(gesvh) = cupdate.payload {
        gesvh
    } else {
        return Err(LTZFError::WrongEndpoint("Gesetzesvorhaben".to_owned()));
    };
    let mut conn = app.pool.get().await?;
    // if there is a uuid supplied, the match is unique and the data is definitely to be updated. If it cannot be updated, an error is returned.

    // if no uuid is supplied, either there was none found, in which case a best-effort match is created with all available data
    // and if more than one match is found, an error is returned and a human intervention is required to decide the case.
    // else if a singular match is found it is handled as if a uuid was supplied

    // if no match at all can be found, a new entry is created
    if gesvh.ext_id.is_some() {
        let mut update_gesvh = dbcon::UpdateGesetzesvorhaben {
            ext_id: gesvh.ext_id,
            ..Default::default()
        };
        let mut matches = dbcon::Gesetzesvorhaben::matches(&mut conn, &update_gesvh).await?;
        // if one match: update
        if matches.is_empty() {
            // do further matching
        }
        else{
            // create
            let gen_id = Uuid::now_v7();
            // construct the struct, check for validity
            // if valid, insert else return error
        }
    }else{
        // do further matching
    }
    // if more than one match: error on ambiguous data, let a human decide
    todo!()
}
