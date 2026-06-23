use herdr_insight_common::{InsightResult, PaneInfo};

/// Abstract data source for pane information.
/// Implemented by infra::herdr_client.
pub trait PaneRepository {
    fn list_all() -> InsightResult<Vec<PaneInfo>>;
}
