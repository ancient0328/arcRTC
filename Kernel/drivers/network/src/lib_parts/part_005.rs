impl TransportSecurityPathGuard {
    /// secure mode が失敗した場合に insecure path へ落ちないことを確認します。
    pub const fn try_new(
        required_secure_mode: bool,
        secure_transport_setup_attempted: bool,
        setup_or_path_failure: bool,
        insecure_fallback_absent: bool,
        failure_mapped_to_cataloged_reason: bool,
        development_only_non_sensitive_canonical_declared: bool,
        development_evidence_not_used_for_production_readiness: bool,
    ) -> Result<Self, TransportSecurityPathError> {
        if required_secure_mode && !secure_transport_setup_attempted {
            return Err(TransportSecurityPathError::SecureSetupMissing);
        }
        if setup_or_path_failure && !insecure_fallback_absent {
            return Err(TransportSecurityPathError::InsecureFallback);
        }
        if setup_or_path_failure && !failure_mapped_to_cataloged_reason {
            return Err(TransportSecurityPathError::FailureReasonMissing);
        }
        if !required_secure_mode
            && (!development_only_non_sensitive_canonical_declared
                || !development_evidence_not_used_for_production_readiness)
        {
            return Err(TransportSecurityPathError::DevelopmentOnlyBoundaryMissing);
        }

        Ok(Self {
            required_secure_mode,
            secure_transport_setup_attempted,
            setup_or_path_failure,
            insecure_fallback_absent,
            failure_mapped_to_cataloged_reason,
            development_only_non_sensitive_canonical_declared,
            development_evidence_not_used_for_production_readiness,
        })
    }
}

/// transport security evidence class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportSecurityEvidenceClass {
    /// configuration shape validation.
    ConfigurationShapeValidation,
    /// secret source availability.
    SecretSourceAvailability,
    /// listener startup.
    ListenerStartup,
    /// peer verification behavior.
    PeerVerificationBehavior,
    /// internal service identity/trust mapping.
    InternalServiceIdentityTrustMapping,
    /// DTLS/SRTP or TLS session establishment.
    SessionEstablishment,
    /// negative test for invalid peer or missing secret.
    NegativeSecurityCase,
    /// redaction of secret material in logs/reports.
    SecretMaterialRedaction,
    /// rotation generation/overlap evidence.
    RotationGenerationOverlap,
    /// secure media session evidence.
    SecureMediaSession,
    /// edge termination and downstream protection evidence.
    EdgeTerminationDownstreamProtection,
}

/// transport security evidence shape です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransportSecurityEvidenceShape {
    evidence_class: TransportSecurityEvidenceClass,
    correlation_or_startup_run_id_recorded: bool,
    configuration_shape_recorded: bool,
    secret_source_reference_recorded_when_claimed: bool,
    listener_startup_not_used_as_peer_verification: bool,
    peer_verification_not_used_as_internal_authorization: bool,
    edge_termination_boundary_declared_when_claimed: bool,
    rotation_state_recorded_when_claimed: bool,
    secure_media_session_not_inferred_from_configuration: bool,
    secret_material_redacted: bool,
}

/// transport security evidence shape の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportSecurityEvidenceShapeError {
    /// evidence に required field が不足しています。
    RequiredEvidenceFieldMissing,
    /// listener startup と peer verification を混同しています。
    ListenerStartupConflatedWithPeerVerification,
    /// peer verification と internal service authorization を混同しています。
    PeerVerificationConflatedWithAuthorization,
    /// edge termination boundary が未宣言です。
    EdgeTerminationBoundaryMissing,
    /// rotation evidence が未記録です。
    RotationEvidenceMissing,
    /// secure media session readiness を configuration から推論しています。
    SecureMediaSessionConflatedWithConfiguration,
    /// secret material redaction がありません。
    SecretRedactionMissing,
}

impl TransportSecurityEvidenceShape {
    /// evidence class 間の流用を禁止します。
    pub const fn try_new(
        evidence_class: TransportSecurityEvidenceClass,
        correlation_or_startup_run_id_recorded: bool,
        configuration_shape_recorded: bool,
        secret_source_reference_recorded_when_claimed: bool,
        listener_startup_not_used_as_peer_verification: bool,
        peer_verification_not_used_as_internal_authorization: bool,
        edge_termination_boundary_declared_when_claimed: bool,
        rotation_state_recorded_when_claimed: bool,
        secure_media_session_not_inferred_from_configuration: bool,
        secret_material_redacted: bool,
    ) -> Result<Self, TransportSecurityEvidenceShapeError> {
        if !correlation_or_startup_run_id_recorded || !configuration_shape_recorded {
            return Err(TransportSecurityEvidenceShapeError::RequiredEvidenceFieldMissing);
        }
        if matches!(
            evidence_class,
            TransportSecurityEvidenceClass::SecretSourceAvailability
                | TransportSecurityEvidenceClass::SessionEstablishment
        ) && !secret_source_reference_recorded_when_claimed
        {
            return Err(TransportSecurityEvidenceShapeError::RequiredEvidenceFieldMissing);
        }
        if !listener_startup_not_used_as_peer_verification {
            return Err(
                TransportSecurityEvidenceShapeError::ListenerStartupConflatedWithPeerVerification,
            );
        }
        if !peer_verification_not_used_as_internal_authorization {
            return Err(
                TransportSecurityEvidenceShapeError::PeerVerificationConflatedWithAuthorization,
            );
        }
        if matches!(
            evidence_class,
            TransportSecurityEvidenceClass::EdgeTerminationDownstreamProtection
        ) && !edge_termination_boundary_declared_when_claimed
        {
            return Err(TransportSecurityEvidenceShapeError::EdgeTerminationBoundaryMissing);
        }
        if matches!(
            evidence_class,
            TransportSecurityEvidenceClass::RotationGenerationOverlap
        ) && !rotation_state_recorded_when_claimed
        {
            return Err(TransportSecurityEvidenceShapeError::RotationEvidenceMissing);
        }
        if !secure_media_session_not_inferred_from_configuration {
            return Err(
                TransportSecurityEvidenceShapeError::SecureMediaSessionConflatedWithConfiguration,
            );
        }
        if !secret_material_redacted {
            return Err(TransportSecurityEvidenceShapeError::SecretRedactionMissing);
        }

        Ok(Self {
            evidence_class,
            correlation_or_startup_run_id_recorded,
            configuration_shape_recorded,
            secret_source_reference_recorded_when_claimed,
            listener_startup_not_used_as_peer_verification,
            peer_verification_not_used_as_internal_authorization,
            edge_termination_boundary_declared_when_claimed,
            rotation_state_recorded_when_claimed,
            secure_media_session_not_inferred_from_configuration,
            secret_material_redacted,
        })
    }
}

/// transport security configuration 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedTransportSecurityConfigurationBehavior {
    /// core reads certificate/key files or secret manager clients.
    CoreReadsTransportSecretMaterial,
    /// driver accepts insecure fallback after required secure mode fails.
    InsecureFallbackAfterSecureModeFailure,
    /// concrete TLS/DTLS/SRTP object crosses into core.
    ConcreteSecurityBackendCrossesCore,
    /// raw key/token/secret appears in audit/log/metric/report.
    RawTransportSecretMaterialExposed,
    /// entrypoint configuration branch changes core security semantics silently.
    EntrypointConfigurationSilentlyChangesCoreSecurityPolicy,
    /// listener startup success is treated as peer verification success.
    ListenerStartupAsPeerVerificationSuccess,
    /// peer verification success is treated as internal service authorization success.
    PeerVerificationAsInternalServiceAuthorization,
    /// stale or revoked transport secret is accepted without rotation policy.
    StaleOrRevokedTransportSecretAccepted,
    /// edge TLS termination is treated as backend secure transport without edge trust policy.
    EdgeTerminationAsBackendSecureTransport,
    /// secure media session readiness is claimed from transport configuration alone.
    SecureMediaReadinessFromConfigurationOnly,
}

fn accept_reference(
    reference: UntrustedReference,
    authority: ReferenceAuthority,
) -> Result<OpaqueReference, TurnWireFailure> {
    OpaqueReference::accept_untrusted(reference, authority).map_err(
        |_error: OpaqueReferenceError| {
            TurnWireFailure::simple(TurnWireFailureKind::MalformedTurnMessage)
        },
    )
}

fn optional_allocation_id(
    reference: Option<UntrustedReference>,
) -> Result<Option<AllocationId>, TurnWireFailure> {
    reference
        .map(|reference| {
            accept_reference(reference, ReferenceAuthority::CoreValidatedUntrustedInput)
                .map(AllocationId::new)
        })
        .transpose()
}

fn optional_permission_id(
    reference: Option<UntrustedReference>,
) -> Result<Option<PermissionId>, TurnWireFailure> {
    reference
        .map(|reference| {
            accept_reference(reference, ReferenceAuthority::CoreValidatedUntrustedInput)
                .map(PermissionId::new)
        })
        .transpose()
}

fn optional_channel_bind_id(
    reference: Option<UntrustedReference>,
) -> Result<Option<ChannelBindId>, TurnWireFailure> {
    reference
        .map(|reference| {
            accept_reference(reference, ReferenceAuthority::CoreValidatedUntrustedInput)
                .map(ChannelBindId::new)
        })
        .transpose()
}

fn optional_credential_ref(
    reference: Option<UntrustedReference>,
) -> Result<Option<CredentialRef>, TurnWireFailure> {
    reference
        .map(|reference| {
            accept_reference(reference, ReferenceAuthority::DriverCredentialConversion)
                .map(CredentialRef::new)
        })
        .transpose()
}

fn optional_packet_id(
    reference: Option<UntrustedReference>,
) -> Result<Option<PacketId>, TurnWireFailure> {
    reference
        .map(|reference| {
            accept_reference(reference, ReferenceAuthority::CoreValidatedUntrustedInput)
                .map(PacketId::new)
        })
        .transpose()
}

fn resource_bound_decision(
    resource: ResourceBoundKind,
    reason_code: &'static str,
    references: ResourceBoundReferenceSet,
) -> ResourceBoundDecision {
    let closed_action = REQUIRED_RESOURCE_BOUND_CLOSED_ACTIONS
        .iter()
        .find(|action| action.resource() == resource && action.reason_code() == reason_code)
        .copied()
        .expect("TURN wire resource bound must be present in core quality catalog");

    ResourceBoundDecision::try_new(closed_action, references)
        .expect("TURN wire resource bound references must satisfy canonical shape")
}
