use std::time::Duration;

use chrono::{DateTime, Utc};
use eyre::Result;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::models::exp::{
    component::{self, Component, ComponentEdit, ComponentQuery},
    project::{
        ProjectComponent, ProjectComponentKind, ProjectQueryContext,
        ProjectQueryRequirements,
    },
};
use crate::models::ids::ProjectId;
use crate::util::error::Context;

#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum EnshroudedVoiceChatMode {
    #[default]
    Proximity,
    Global,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum EnshroudedServerRole {
    Admin,
    Friend,
    Guest,
    Visitor,
    Custom,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum EnshroudedPasswordVisibility {
    #[default]
    None,
    Required,
    ContactOwner,
    Public,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct EnshroudedUserGroup {
    #[validate(length(min = 1, max = 64))]
    pub name: String,
    pub role: EnshroudedServerRole,
    pub can_kick_ban: bool,
    pub can_access_inventories: bool,
    pub can_edit_world: bool,
    pub can_edit_base: bool,
    pub can_extend_base: bool,
    #[validate(range(max = 16))]
    pub reserved_slots: u8,
    #[serde(default)]
    pub password_visibility: EnshroudedPasswordVisibility,
}

fn default_user_groups() -> Vec<EnshroudedUserGroup> {
    vec![
        EnshroudedUserGroup {
            name: "Admin".into(),
            role: EnshroudedServerRole::Admin,
            can_kick_ban: true,
            can_access_inventories: true,
            can_edit_world: true,
            can_edit_base: true,
            can_extend_base: true,
            reserved_slots: 4,
            password_visibility: EnshroudedPasswordVisibility::ContactOwner,
        },
        EnshroudedUserGroup {
            name: "Friend".into(),
            role: EnshroudedServerRole::Friend,
            can_kick_ban: false,
            can_access_inventories: true,
            can_edit_world: true,
            can_edit_base: true,
            can_extend_base: true,
            reserved_slots: 0,
            password_visibility: EnshroudedPasswordVisibility::ContactOwner,
        },
        EnshroudedUserGroup {
            name: "Guest".into(),
            role: EnshroudedServerRole::Guest,
            can_kick_ban: false,
            can_access_inventories: false,
            can_edit_world: true,
            can_edit_base: false,
            can_extend_base: false,
            reserved_slots: 0,
            password_visibility: EnshroudedPasswordVisibility::Required,
        },
        EnshroudedUserGroup {
            name: "Visitor".into(),
            role: EnshroudedServerRole::Visitor,
            can_kick_ban: false,
            can_access_inventories: false,
            can_edit_world: false,
            can_edit_base: false,
            can_extend_base: false,
            reserved_slots: 0,
            password_visibility: EnshroudedPasswordVisibility::Required,
        },
    ]
}

component::define! {
    /// An Enshrouded mod project.
    #[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
    pub struct ModProject {}

    /// A World Editor export containing a reusable part of an Enshrouded world.
    #[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
    pub struct SchematicProject {}
}

impl ProjectComponent for ModProject {
    fn kind() -> ProjectComponentKind {
        ProjectComponentKind::EnshroudedMod
    }
}

impl ProjectComponent for SchematicProject {
    fn kind() -> ProjectComponentKind {
        ProjectComponentKind::Schematic
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct EnshroudedServerProject {
    /// Public IPv4 address or DNS name, without a port.
    #[validate(length(max = 255))]
    pub address: String,
    /// UDP query port configured by the Enshrouded dedicated server.
    pub query_port: u16,
    /// Geographical region in which the server is hosted.
    #[serde(default)]
    pub region: Option<String>,
    /// Languages preferred by the server community.
    #[serde(default)]
    pub languages: Vec<String>,
    #[serde(default)]
    pub voice_chat_enabled: bool,
    #[serde(default)]
    pub voice_chat_mode: EnshroudedVoiceChatMode,
    #[serde(default)]
    pub text_chat_enabled: bool,
    #[serde(default = "default_user_groups")]
    #[validate(nested)]
    pub user_groups: Vec<EnshroudedUserGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct EnshroudedServerProjectEdit {
    #[validate(length(max = 255))]
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub query_port: Option<u16>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub languages: Option<Vec<String>>,
    #[serde(default)]
    pub voice_chat_enabled: Option<bool>,
    #[serde(default)]
    pub voice_chat_mode: Option<EnshroudedVoiceChatMode>,
    #[serde(default)]
    pub text_chat_enabled: Option<bool>,
    #[serde(default)]
    #[validate(nested)]
    pub user_groups: Option<Vec<EnshroudedUserGroup>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct EnshroudedServerProjectQuery {
    pub address: String,
    pub query_port: u16,
    pub region: Option<String>,
    pub languages: Vec<String>,
    pub voice_chat_enabled: bool,
    pub voice_chat_mode: EnshroudedVoiceChatMode,
    pub text_chat_enabled: bool,
    pub user_groups: Vec<EnshroudedUserGroup>,
    pub ping: Option<EnshroudedServerPing>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct EnshroudedServerPing {
    pub when: DateTime<Utc>,
    pub address: String,
    pub query_port: u16,
    pub data: Option<EnshroudedServerPingData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct EnshroudedServerPingData {
    pub latency: Duration,
    pub name: String,
    pub game_version: String,
    pub map: String,
    pub players_online: u8,
    pub players_max: u8,
    pub password_protected: bool,
}

impl ProjectComponent for EnshroudedServerProject {
    fn kind() -> ProjectComponentKind {
        ProjectComponentKind::EnshroudedServer
    }
}

impl Component for EnshroudedServerProject {
    type EntityId = ProjectId;
    type Query = EnshroudedServerProjectQuery;
    type Edit = EnshroudedServerProjectEdit;
}

impl ComponentQuery for EnshroudedServerProjectQuery {
    type Component = EnshroudedServerProject;
    type Requirements = ProjectQueryRequirements;
    type Context = ProjectQueryContext;

    fn collect_requirements(
        _serial: &Self::Component,
        project_id: ProjectId,
        requirements: &mut ProjectQueryRequirements,
    ) {
        requirements.enshrouded_server_pings.insert(project_id);
    }

    fn populate(
        serial: Self::Component,
        project_id: ProjectId,
        context: &ProjectQueryContext,
    ) -> Result<Self> {
        Ok(Self {
            address: serial.address,
            query_port: serial.query_port,
            region: serial.region,
            languages: serial.languages,
            voice_chat_enabled: serial.voice_chat_enabled,
            voice_chat_mode: serial.voice_chat_mode,
            text_chat_enabled: serial.text_chat_enabled,
            user_groups: serial.user_groups,
            ping: context.enshrouded_server_pings.get(&project_id).cloned(),
        })
    }
}

impl ComponentEdit for EnshroudedServerProjectEdit {
    type Component = EnshroudedServerProject;

    fn create(self) -> Result<Self::Component> {
        let query_port = self.query_port.unwrap_or(15637);
        eyre::ensure!(query_port != 0, "`query_port` must not be zero");

        let user_groups = self.user_groups.unwrap_or_else(default_user_groups);
        validate_user_groups(&user_groups)?;

        Ok(EnshroudedServerProject {
            address: self.address.wrap_err("missing `address`")?,
            query_port,
            region: self.region,
            languages: self.languages.unwrap_or_default(),
            voice_chat_enabled: self.voice_chat_enabled.unwrap_or(false),
            voice_chat_mode: self.voice_chat_mode.unwrap_or_default(),
            text_chat_enabled: self.text_chat_enabled.unwrap_or(false),
            user_groups,
        })
    }

    async fn apply_to(self, component: &mut Self::Component) -> Result<()> {
        if let Some(address) = self.address {
            component.address = address;
        }
        if let Some(query_port) = self.query_port {
            eyre::ensure!(query_port != 0, "`query_port` must not be zero");
            component.query_port = query_port;
        }
        if let Some(region) = self.region {
            component.region = Some(region);
        }
        if let Some(languages) = self.languages {
            component.languages = languages;
        }
        if let Some(voice_chat_enabled) = self.voice_chat_enabled {
            component.voice_chat_enabled = voice_chat_enabled;
        }
        if let Some(voice_chat_mode) = self.voice_chat_mode {
            component.voice_chat_mode = voice_chat_mode;
        }
        if let Some(text_chat_enabled) = self.text_chat_enabled {
            component.text_chat_enabled = text_chat_enabled;
        }
        if let Some(user_groups) = self.user_groups {
            validate_user_groups(&user_groups)?;
            component.user_groups = user_groups;
        }
        Ok(())
    }
}

fn validate_user_groups(groups: &[EnshroudedUserGroup]) -> Result<()> {
    eyre::ensure!(
        groups.len() <= 32,
        "no more than 32 user groups are allowed"
    );
    let mut names = std::collections::HashSet::new();
    for group in groups {
        let name = group.name.trim().to_lowercase();
        eyre::ensure!(!name.is_empty(), "user group names must not be empty");
        eyre::ensure!(names.insert(name), "user group names must be unique");
        eyre::ensure!(
            group.reserved_slots <= 16,
            "`reserved_slots` must not exceed 16"
        );
        eyre::ensure!(
            group.password_visibility != EnshroudedPasswordVisibility::Public
                || (group.role != EnshroudedServerRole::Admin
                    && !group.can_kick_ban),
            "admin passwords cannot be public"
        );
    }
    Ok(())
}
