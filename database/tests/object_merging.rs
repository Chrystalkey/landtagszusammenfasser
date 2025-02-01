#![cfg(test)]
// this module sets up a default test dataset

#[test]
fn one_match_nochange(){
    // expects one match
    // resends the same data (with different api_id)
    // expects no change in the database
}

#[test]
fn one_match_diffdata(){
    // expects one match and differing data on the top level
}

#[test]
fn one_match_different_stations(){
    // expects one match and a different set of stations
    // expects no new gsvh and a union of the stations
}

#[test]
fn one_match_less_stations(){
    // expects one match and a subset of the original stations
    // expects no change to the data
}

#[test]
fn one_match_d_station_data(){
    // expects one match and a set of stations with one station having different data in it
    // expects no new gsvh and the updated station to be added
}

#[test]
fn one_match_documents(){
    // expects one match and a different set of documents for a station
}