/// Handles merging of two datasets.
/// in particular, stellungnahme & dokument are atomic.
/// station and gsvh are not in the sense that gsvh.stations and station.stellungnahmen are appendable and deletable.
/// This means the merge strategy is in general to:
/// 1. find a gsvh that is matching enough
///     a. if found exactly one, update the gsvh, see 2.
///     b. if found more than one, send a mail to the admins to select one
///     c. if found none, create a new gsvh, return
/// 2. if a., then update the gsvh properties
/// 3. for each station in the new gsvh, find a matching station
///     a. if found exactly one, update it, see 4.
///     b. if found more than one, send a mail to the admins to select one
///     c. if found none, create a new station & insert
/// 4. if a., then update station properties
/// 5. for each stellungnahme in the new station, find a matching stellungnahme
///    a. if found exactly one, replace it
///    b. if found more than one, send a mail to the admins to select one
///    c. if found none, create a new stellungnahme & insert