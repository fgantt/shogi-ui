use crate::evaluation::material::{
    MaterialEvaluationConfig, MaterialValueSet, MaterialValueSetError,
};
use std::fs::File;
use std::path::Path;

pub struct MaterialValueLoader;

impl MaterialValueLoader {
    pub fn load(
        config: &MaterialEvaluationConfig,
    ) -> Result<MaterialValueSet, MaterialValueSetError> {
        if let Some(path) = config.values_path.as_ref() {
            return MaterialValueSet::from_path(Path::new(path));
        }
        if config.use_research_values {
            Ok(MaterialValueSet::research())
        } else {
            Ok(MaterialValueSet::classic())
        }
    }

    pub fn save(value_set: &MaterialValueSet, path: &Path) -> Result<(), MaterialValueSetError> {
        let file = File::create(path).map_err(|err| MaterialValueSetError::Io {
            path: path.to_path_buf(),
            message: err.to_string(),
        })?;
        value_set.to_writer(file)
    }
}
