/// Narrative cluster records
#[derive(Debug, Clone, Default, sqlx::FromRow)]
pub struct ClusterCount {
    /// The term that describes a token narrative/theme
    pub cluster: String,
    pub bucket: String,
    /// Number of tokens created matching the narrative
    pub count: i32,
}
