//! regulated は optional domain support の独立 surface です。
//!
//! generic communication core には混ぜず、許可された opaque communication reference、
//! audit pointer、non-sensitive tag だけを regulated-local enrichment に使います。

use arcrtc_core_identity::{AuditEventId, CorrelationId, ParticipantId, RoomId};

/// regulated package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegulatedSurface;

/// regulated が所有できる optional support class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegulatedOptionalSupportClass {
    /// domain-specific enrichment record です。
    DomainSpecificEnrichment,
    /// audit pointer mapping helper です。
    AuditPointerMapping,
    /// non-sensitive tag classification です。
    NonSensitiveTagClassification,
    /// external compliance integration helper です。
    ExternalComplianceIntegrationHelper,
}

impl RegulatedOptionalSupportClass {
    /// generic core の accept/reject decision を所有しないことを示します。
    pub const fn owns_core_decision(self) -> bool {
        match self {
            Self::DomainSpecificEnrichment
            | Self::AuditPointerMapping
            | Self::NonSensitiveTagClassification
            | Self::ExternalComplianceIntegrationHelper => false,
        }
    }
}

/// regulated 境界で参照を許される core-owned opaque reference です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RegulatedCommunicationReference {
    /// command / event chain の correlation reference です。
    Correlation(CorrelationId),
    /// room を opaque communication reference として参照します。
    Room(RoomId),
    /// participant を opaque communication reference として参照します。
    Participant(ParticipantId),
    /// core audit event pointer です。
    AuditEvent(AuditEventId),
    /// audit hash-chain record pointer です。
    HashChainRecord(RegulatedHashChainRecordPointer),
}

impl RegulatedCommunicationReference {
    /// この reference が regulated-local enrichment から core を mutate しないことを示します。
    pub const fn is_immutable_pointer_only(&self) -> bool {
        match self {
            Self::Correlation(_)
            | Self::Room(_)
            | Self::Participant(_)
            | Self::AuditEvent(_)
            | Self::HashChainRecord(_) => true,
        }
    }
}

/// regulated-local に保持する audit hash-chain pointer です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RegulatedHashChainRecordPointer {
    value: String,
}

impl RegulatedHashChainRecordPointer {
    /// hash-chain pointer を regulated-local opaque pointer として受理します。
    pub fn accept(value: impl Into<String>) -> Result<Self, RegulatedEnrichmentFailure> {
        let value = value.into();
        if value.is_empty() {
            return Err(RegulatedEnrichmentFailure::new(
                RegulatedEnrichmentFailureKind::EmptyHashChainPointer,
            ));
        }
        if value.chars().any(char::is_control) {
            return Err(RegulatedEnrichmentFailure::new(
                RegulatedEnrichmentFailureKind::ControlCharacterInHashChainPointer,
            ));
        }

        Ok(Self { value })
    }

    /// opaque pointer value です。domain payload として解釈してはいけません。
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

/// regulated-local の closed non-sensitive tag vocabulary です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegulatedNonSensitiveTag {
    /// communication event reference の分類です。
    CommunicationEventReference,
    /// audit pointer reference の分類です。
    AuditPointerReference,
    /// quality metric reference の分類です。
    QualityMetricReference,
    /// compliance mapping reference の分類です。
    ComplianceMappingReference,
}

impl RegulatedNonSensitiveTag {
    /// core required field にはならない regulated-local tag です。
    pub const fn is_core_required(self) -> bool {
        match self {
            Self::CommunicationEventReference
            | Self::AuditPointerReference
            | Self::QualityMetricReference
            | Self::ComplianceMappingReference => false,
        }
    }

    /// core Signaling / SFU / TURN behavior の判断には使いません。
    pub const fn drives_core_behavior(self) -> bool {
        match self {
            Self::CommunicationEventReference
            | Self::AuditPointerReference
            | Self::QualityMetricReference
            | Self::ComplianceMappingReference => false,
        }
    }
}

/// regulated enrichment lifecycle の段階です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegulatedEnrichmentLifecycleStage {
    /// core から許可済み opaque pointer を受け取った段階です。
    ReceivedAllowedCorePointer,
    /// generic core の外側で domain context へ mapping する段階です。
    MappedOutsideGenericCore,
    /// regulated-local enrichment record を生成する段階です。
    LocalRecordEmitted,
    /// immutable core audit pointer だけを参照する段階です。
    ImmutableAuditPointerReferenced,
}

impl RegulatedEnrichmentLifecycleStage {
    /// core accept/reject decision には参加しない post-core stage です。
    pub const fn is_post_core_optional(self) -> bool {
        match self {
            Self::ReceivedAllowedCorePointer
            | Self::MappedOutsideGenericCore
            | Self::LocalRecordEmitted
            | Self::ImmutableAuditPointerReferenced => true,
        }
    }
}

/// regulated enrichment が core decision に関与するかどうかの閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegulatedCoreDecisionParticipation {
    /// core decision には参加しません。
    NeverParticipates,
    /// 禁止: core decision を要求しています。
    RequiresCoreDecision,
}

/// regulated enrichment が core audit event を変更するかどうかの閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegulatedCoreAuditMutation {
    /// immutable pointer として参照するだけです。
    ImmutableReferenceOnly,
    /// 禁止: core audit event の変更を要求しています。
    MutatesCoreAuditEvent,
}

/// domain payload が generic core の必須入力になるかどうかの閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegulatedDomainPayloadRequirement {
    /// generic core には要求しません。
    NotRequiredByGenericCore,
    /// 禁止: generic core の必須入力にしています。
    RequiredByGenericCore,
}

/// regulated optional enrichment の入力です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RegulatedEnrichmentInput {
    support_class: RegulatedOptionalSupportClass,
    lifecycle_stage: RegulatedEnrichmentLifecycleStage,
    reference: RegulatedCommunicationReference,
    audit_pointer: Option<AuditEventId>,
    tag: Option<RegulatedNonSensitiveTag>,
    core_decision_participation: RegulatedCoreDecisionParticipation,
    core_audit_mutation: RegulatedCoreAuditMutation,
    domain_payload_requirement: RegulatedDomainPayloadRequirement,
}

impl RegulatedEnrichmentInput {
    /// regulated optional enrichment の入力を作ります。
    pub const fn new(
        support_class: RegulatedOptionalSupportClass,
        lifecycle_stage: RegulatedEnrichmentLifecycleStage,
        reference: RegulatedCommunicationReference,
        audit_pointer: Option<AuditEventId>,
        tag: Option<RegulatedNonSensitiveTag>,
        core_decision_participation: RegulatedCoreDecisionParticipation,
        core_audit_mutation: RegulatedCoreAuditMutation,
        domain_payload_requirement: RegulatedDomainPayloadRequirement,
    ) -> Self {
        Self {
            support_class,
            lifecycle_stage,
            reference,
            audit_pointer,
            tag,
            core_decision_participation,
            core_audit_mutation,
            domain_payload_requirement,
        }
    }

    /// support class です。
    pub const fn support_class(&self) -> RegulatedOptionalSupportClass {
        self.support_class
    }

    /// lifecycle stage です。
    pub const fn lifecycle_stage(&self) -> RegulatedEnrichmentLifecycleStage {
        self.lifecycle_stage
    }

    /// allowed opaque reference です。
    pub const fn reference(&self) -> &RegulatedCommunicationReference {
        &self.reference
    }

    /// optional audit pointer です。
    pub const fn audit_pointer(&self) -> Option<&AuditEventId> {
        self.audit_pointer.as_ref()
    }

    /// optional regulated-local tag です。
    pub const fn tag(&self) -> Option<RegulatedNonSensitiveTag> {
        self.tag
    }
}

/// guard 通過後の regulated-local enrichment record です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RegulatedEnrichmentRecord {
    input: RegulatedEnrichmentInput,
    record_class: RegulatedOptionalSupportClass,
}

impl RegulatedEnrichmentRecord {
    /// 元入力です。
    pub const fn input(&self) -> &RegulatedEnrichmentInput {
        &self.input
    }

    /// regulated-local record class です。
    pub const fn record_class(&self) -> RegulatedOptionalSupportClass {
        self.record_class
    }

    /// core audit event を置換しない local record です。
    pub const fn replaces_core_audit_event(&self) -> bool {
        false
    }
}

/// regulated enrichment admission guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegulatedEnrichmentGuard;

impl RegulatedEnrichmentGuard {
    /// input が optional regulated enrichment の境界内にある場合だけ record 化します。
    pub fn admit(
        input: RegulatedEnrichmentInput,
    ) -> Result<RegulatedEnrichmentRecord, RegulatedEnrichmentFailure> {
        if input.support_class.owns_core_decision() {
            return Err(RegulatedEnrichmentFailure::new(
                RegulatedEnrichmentFailureKind::SupportClassOwnsCoreDecision,
            ));
        }
        if !input.lifecycle_stage.is_post_core_optional() {
            return Err(RegulatedEnrichmentFailure::new(
                RegulatedEnrichmentFailureKind::LifecycleNotPostCoreOptional,
            ));
        }
        if !input.reference.is_immutable_pointer_only() {
            return Err(RegulatedEnrichmentFailure::new(
                RegulatedEnrichmentFailureKind::ReferenceIsNotImmutablePointer,
            ));
        }
        if let Some(tag) = input.tag {
            if tag.is_core_required() {
                return Err(RegulatedEnrichmentFailure::new(
                    RegulatedEnrichmentFailureKind::TagRequiredByGenericCore,
                ));
            }
            if tag.drives_core_behavior() {
                return Err(RegulatedEnrichmentFailure::new(
                    RegulatedEnrichmentFailureKind::TagDrivesCoreBehavior,
                ));
            }
        }
        if matches!(
            input.core_decision_participation,
            RegulatedCoreDecisionParticipation::RequiresCoreDecision
        ) {
            return Err(RegulatedEnrichmentFailure::new(
                RegulatedEnrichmentFailureKind::CoreDecisionParticipationRequested,
            ));
        }
        if matches!(
            input.core_audit_mutation,
            RegulatedCoreAuditMutation::MutatesCoreAuditEvent
        ) {
            return Err(RegulatedEnrichmentFailure::new(
                RegulatedEnrichmentFailureKind::CoreAuditMutationRequested,
            ));
        }
        if matches!(
            input.domain_payload_requirement,
            RegulatedDomainPayloadRequirement::RequiredByGenericCore
        ) {
            return Err(RegulatedEnrichmentFailure::new(
                RegulatedEnrichmentFailureKind::DomainPayloadRequiredByGenericCore,
            ));
        }

        let record_class = input.support_class;
        Ok(RegulatedEnrichmentRecord {
            input,
            record_class,
        })
    }
}

/// dependency boundary の source layer です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegulatedDependencySource {
    /// core layer です。
    Core,
    /// drivers layer です。
    Drivers,
    /// entrypoints layer です。
    Entrypoints,
    /// sdk layer です。
    Sdk,
    /// regulated layer です。
    Regulated,
}

/// dependency boundary の target layer です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegulatedDependencyTarget {
    /// allowed core opaque identity references です。
    CoreOpaqueIdentityReferences,
    /// generic core semantics です。
    CoreProtocolSemantics,
    /// drivers layer です。
    Drivers,
    /// entrypoints layer です。
    Entrypoints,
    /// sdk layer です。
    Sdk,
    /// regulated layer です。
    Regulated,
}

/// regulated boundary の dependency guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegulatedDependencyGuard;

impl RegulatedDependencyGuard {
    /// `regulated -> core opaque references` だけを admitted とします。
    pub const fn admits(
        source: RegulatedDependencySource,
        target: RegulatedDependencyTarget,
    ) -> bool {
        matches!(
            (source, target),
            (
                RegulatedDependencySource::Regulated,
                RegulatedDependencyTarget::CoreOpaqueIdentityReferences
            )
        )
    }
}

/// regulated 境界で禁止する behavior です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedRegulatedBehavior {
    /// core が regulated type を import します。
    CoreImportsRegulatedType,
    /// sdk が regulated に依存します。
    SdkDependsOnRegulated,
    /// regulated が sdk に依存します。
    RegulatedDependsOnSdk,
    /// regulated が drivers に依存します。
    RegulatedDependsOnDrivers,
    /// regulated が entrypoints に依存します。
    RegulatedDependsOnEntrypoints,
    /// regulated が Signaling / SFU / TURN decision を所有します。
    RegulatedOwnsCommunicationProtocolDecision,
    /// sensitive domain payload を core event に要求します。
    SensitiveDomainPayloadRequiredByCoreEvent,
    /// regulated enrichment を generic core operation の必須経路にします。
    GenericCoreRequiresRegulatedEnrichment,
    /// regulated enrichment が core audit event を変更します。
    RegulatedMutatesCoreAuditEvent,
    /// regulated tag を hidden core authorization policy にします。
    RegulatedTagBecomesHiddenCoreAuthorizationPolicy,
    /// driver buffer / packet lease を regulated に渡します。
    DriverBufferOrPacketLeaseReference,
    /// raw token / credential を regulated pointer として扱います。
    RawCredentialReference,
    /// raw media payload を regulated pointer として扱います。
    RawMediaPayloadReference,
}

/// regulated enrichment failure mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegulatedEnrichmentFailureKind {
    /// support class が core decision ownership を要求しました。
    SupportClassOwnsCoreDecision,
    /// lifecycle が post-core optional ではありません。
    LifecycleNotPostCoreOptional,
    /// reference が immutable pointer only ではありません。
    ReferenceIsNotImmutablePointer,
    /// tag が generic core required field になっています。
    TagRequiredByGenericCore,
    /// tag が core behavior を決めています。
    TagDrivesCoreBehavior,
    /// core decision participation を要求しました。
    CoreDecisionParticipationRequested,
    /// core audit event mutation を要求しました。
    CoreAuditMutationRequested,
    /// domain payload が generic core required field になっています。
    DomainPayloadRequiredByGenericCore,
    /// hash-chain pointer が空です。
    EmptyHashChainPointer,
    /// hash-chain pointer に制御文字があります。
    ControlCharacterInHashChainPointer,
}

impl RegulatedEnrichmentFailureKind {
    /// reason catalog に接続できる stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::SupportClassOwnsCoreDecision => "regulated_support_class_owns_core_decision",
            Self::LifecycleNotPostCoreOptional => "regulated_lifecycle_not_post_core_optional",
            Self::ReferenceIsNotImmutablePointer => "regulated_reference_not_immutable_pointer",
            Self::TagRequiredByGenericCore => "regulated_tag_required_by_generic_core",
            Self::TagDrivesCoreBehavior => "regulated_tag_drives_core_behavior",
            Self::CoreDecisionParticipationRequested => {
                "regulated_core_decision_participation_requested"
            }
            Self::CoreAuditMutationRequested => "regulated_core_audit_mutation_requested",
            Self::DomainPayloadRequiredByGenericCore => {
                "regulated_domain_payload_required_by_generic_core"
            }
            Self::EmptyHashChainPointer => "regulated_empty_hash_chain_pointer",
            Self::ControlCharacterInHashChainPointer => {
                "regulated_control_character_in_hash_chain_pointer"
            }
        }
    }
}

/// regulated enrichment failure です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegulatedEnrichmentFailure {
    kind: RegulatedEnrichmentFailureKind,
}

impl RegulatedEnrichmentFailure {
    /// failure を作ります。
    pub const fn new(kind: RegulatedEnrichmentFailureKind) -> Self {
        Self { kind }
    }

    /// failure kind です。
    pub const fn kind(self) -> RegulatedEnrichmentFailureKind {
        self.kind
    }

    /// reason catalog に接続できる stable code です。
    pub const fn reason_code(self) -> &'static str {
        self.kind.reason_code()
    }
}
