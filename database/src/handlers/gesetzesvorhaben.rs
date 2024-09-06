use std::sync::Arc;

use crate::error::*;
use crate::infra::api as ifapi;
use crate::infra::db::connection as dbcon;
use crate::AppState;
use ifapi::{CUPPayload, CUPResponse, CUPResponsePayload, DatabaseInteraction};
use uuid::Uuid;
macro_rules! handle_retrieval_error {
    ($field:expr, $name:expr, $conn:ident, $app:ident) => {
        match &$field {
            Some(thing) => match thing.fetch_id(&mut $conn).await {
                Ok(id) => Some(id),
                Err(LTZFError::RetrievalError(RetrievalError::NoMatch)) => {
                    crate::external::no_match_found(
                        format!(
                            "No match was found for field `{}` using this query: {:?}",
                            $name, &$field
                        ),
                        $app.clone(),
                    )
                    .await;
                    None
                }
                Err(error) => return Err(error),
            },
            None => None,
        }
    };
}
pub(crate) async fn handle_gesvh(
    app: Arc<AppState>,
    cupdate: ifapi::CUPUpdate,
) -> std::result::Result<CUPResponse, LTZFError> {
    let gesvh = if let CUPPayload::GesVH(gesvh) = cupdate.payload {
        gesvh
    } else {
        return Err(LTZFError::WrongEndpoint("Gesetzesvorhaben".to_owned()));
    };
    let mut conn = app.pool.get().await.map_err(DatabaseError::from)?;
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
        let matches = dbcon::gesetzesvorhaben::select_matching(&mut conn, update_gesvh)
            .await
            .map_err(DatabaseError::from)?;
        // if one match: update
        if matches.is_empty() {
            // could not find anything to update.
            // this means the collectors knows of an object that does not exist in the database. This is an error to be returned.
        } else {
            // update
        }
    } else {
        // no id supplied, so assumed create was intended
        let gen_id = Uuid::now_v7();
        let feder = handle_retrieval_error!(gesvh.federfuehrung, "Federführung", conn, app);
        let initiat = handle_retrieval_error!(gesvh.initiator, "Initiator", conn, app);

        let ins_gesvh = dbcon::gesetzesvorhaben::Insert {
            ext_id: gen_id,
            off_titel: gesvh.off_titel.map_or(
                Err(DatabaseError::DatabaseError(
                    "off_titel is a required field".to_owned(),
                )),
                |x| Ok(x),
            )?,
            titel: gesvh.titel.map_or(
                Err(DatabaseError::DatabaseError(
                    "Titel is a required field".to_owned(),
                )),
                |x| Ok(x),
            )?,
            verfassungsaendernd: gesvh.verfassungsaendernd.map_or(
                Err(DatabaseError::DatabaseError(
                    "Verfassungsändernd is a required field".to_owned(),
                )),
                |x| Ok(x),
            )?,
            id_gesblatt: gesvh.id_gesblatt,
            url_gesblatt: gesvh.url_gesblatt,
            trojaner: gesvh.trojaner,
            feder: feder,
            initiat,
        };
        // construct the struct, check for validity
        let result = dbcon::gesetzesvorhaben::insert(&mut conn, ins_gesvh)
            .await
            .map_err(DatabaseError::from)?;
        if result == 0 {
            return Err(DatabaseError::DatabaseError("Insert failed".to_owned()).into());
        } else {
            let response = CUPResponse {
                msg_id: Uuid::now_v7(),
                timestamp: chrono::Utc::now(),
                responding_to: cupdate.msg_id,
                payload: CUPResponsePayload {
                    data: CUPPayload::GesVH(ifapi::Gesetzesvorhaben {
                        ext_id: Some(gen_id),
                        ..Default::default()
                    }),
                    state: ifapi::CUPRessourceState::Created,
                },
            };
            return Ok(response);
        };
    }
    // if more than one match: error on ambiguous data, let a human decide
    todo!();
}
