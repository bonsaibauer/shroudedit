use crate::models::projects::Project;

use super::{ProjectNag, ProjectNagKind, ProjectNagSeverity};

const MAX_LANGUAGE_COUNT: usize = 10;

pub(super) fn validate(project: &Project) -> Vec<ProjectNag> {
    let mut nags = Vec::new();
    let server = project.components.enshrouded_server.as_ref();

    if server.is_some_and(|server| server.region.is_none()) {
        nags.push(ProjectNag::new(
            ProjectNagKind::SelectCountry,
            ProjectNagSeverity::Required,
        ));
    }

    if server.is_some_and(|server| server.address.trim().is_empty()) {
        nags.push(ProjectNag::new(
            ProjectNagKind::AddJavaAddress,
            ProjectNagSeverity::Required,
        ));
    }

    if let Some(language_count) = server
        .map(|server| server.languages.len())
        .filter(|language_count| *language_count > MAX_LANGUAGE_COUNT)
    {
        nags.push(
            ProjectNag::new(
                ProjectNagKind::TooManyLanguages,
                ProjectNagSeverity::Warning,
            )
            .with_details(serde_json::json!({
                "language_count": language_count,
            })),
        );
    }

    if server.is_some_and(|server| server.languages.is_empty()) {
        nags.push(ProjectNag::new(
            ProjectNagKind::SelectLanguage,
            ProjectNagSeverity::Suggestion,
        ));
    }

    nags
}
