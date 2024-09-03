use std::sync::Arc;

use crate::infra::api::{CUPPayload, CUPResponse, CUPResponsePayload};
use crate::infra::db::connection as dbcon;
use crate::{error::LTZFError, infra::api as ifapi, AppState};
use axum::{extract::State, Json};
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
        let update_gesvh = dbcon::gesetzesvorhaben::Update {
            ext_id: gesvh.ext_id,
            ..Default::default()
        };
        let matches = dbcon::gesetzesvorhaben::select_matching(&mut conn, update_gesvh).await?;
        // if one match: update
        if matches.is_empty() {
            // create
            let gen_id = Uuid::now_v7();
            let ins_gesvh = dbcon::gesetzesvorhaben::Insert {
                ext_id: gen_id,
                off_titel: gesvh
                    .off_titel
                    .expect("Offizieller Titel is a required field"),
                titel: gesvh.titel.expect("Titel is a required field"),
                verfassungsaendernd: gesvh
                    .verfassungsaendernd
                    .expect("Verfassungsändernd is a required field"),
                id_gesblatt: gesvh.id_gesblatt,
                url_gesblatt: gesvh.url_gesblatt,
                trojaner: gesvh.trojaner,
                feder: None, // TODO: implement search for associated fields
                initiat: None,
            };
            // construct the struct, check for validity
            let result = dbcon::gesetzesvorhaben::insert(&mut conn, ins_gesvh).await?;
            if result == 0 {
                return Err(LTZFError::DatabaseError("Insert failed".to_owned()));
            } else {
                let response = CUPResponse {
                    msg_id: Uuid::now_v7(),
                    timestamp: chrono::Utc::now(),
                    responding_to: cupdate.msg_id,
                    payload: CUPResponsePayload {
                        data: CUPPayload::GesVH(ifapi::updateable_entities::Gesetzesvorhaben {
                            ext_id: Some(gen_id),
                            ..Default::default()
                        }),
                        state: ifapi::CUPRessourceState::Created,
                    },
                };
                return Ok(Json(response));
            };
        } else {
            // update
        }
    } else {
        // do extensive matching
    }
    // if more than one match: error on ambiguous data, let a human decide
    todo!();
}
