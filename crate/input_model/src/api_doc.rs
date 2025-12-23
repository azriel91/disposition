use disposition_model_common::{
    edge::{EdgeGroupId, EdgeId},
    theme::Css,
    Id,
};

use crate::{
    edge::EdgeKind,
    entity::{EntityDescs, EntityType, EntityTypeId, EntityTypes},
    process::{
        ProcessDiagram, ProcessId, ProcessStepId, ProcessSteps, Processes, StepDescs,
        StepThingInteractions,
    },
    tag::{TagId, TagNames, TagThings},
    theme::{
        CssClassPartials, IdOrDefaults, StyleAlias, StyleAliases, TagIdOrDefaults, ThemeAttr,
        ThemeDefault, ThemeStyles, ThemeTagThingsFocus, ThemeThingDependenciesStyles,
        ThemeTypesStyles, ThingsFocusStyles,
    },
    thing::{
        ThingCopyText, ThingDependencies, ThingHierarchy, ThingId, ThingInteractions, ThingNames,
    },
    InputDiagram,
};

/// Structure that represents the OpenAPI documentation for the Disposition API.
#[derive(utoipa::OpenApi)]
#[openapi(components(schemas(
    Css,
    CssClassPartials,
    EdgeGroupId,
    EdgeId,
    EdgeKind,
    EntityDescs,
    EntityType,
    EntityTypeId,
    EntityTypes,
    Id,
    IdOrDefaults,
    InputDiagram,
    TagId,
    TagIdOrDefaults,
    TagNames,
    TagThings,
    ProcessDiagram,
    ProcessId,
    ProcessStepId,
    ProcessSteps,
    Processes,
    StepDescs,
    StepThingInteractions,
    StyleAlias,
    StyleAliases,
    ThemeAttr,
    ThemeDefault,
    ThemeStyles,
    ThemeTagThingsFocus,
    ThemeThingDependenciesStyles,
    ThemeTypesStyles,
    ThingCopyText,
    ThingDependencies,
    ThingHierarchy,
    ThingId,
    ThingInteractions,
    ThingNames,
    ThingsFocusStyles,
)))]
pub struct ApiDoc;
