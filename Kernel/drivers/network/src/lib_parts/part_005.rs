impl TransportSecurityPathGuard {
    /// secure mode が失敗した場合に insecure path へ落ちないことを確認します。
    pub const fn try_new(
        required_secure_mode: bool,
        secure_transport_setup_attempted: bool,
        setup_or_path_failure: bool,
        insecure_fallback_absent: bool,
        failure_mapped_to_cataloged_reason: bool,
        development_only_non_sensitive_policy_enabled: bool,
        development_transport_not_used_as_secure_runtime: bool,
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
            && (!development_only_non_sensitive_policy_enabled
                || !development_transport_not_used_as_secure_runtime)
        {
            return Err(TransportSecurityPathError::DevelopmentOnlyBoundaryMissing);
        }

        Ok(Self {
            required_secure_mode,
            secure_transport_setup_attempted,
            setup_or_path_failure,
            insecure_fallback_absent,
            failure_mapped_to_cataloged_reason,
            development_only_non_sensitive_policy_enabled,
            development_transport_not_used_as_secure_runtime,
        })
    }
}

/// transport security runtime verification class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportSecurityVerificationClass {
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
    /// redaction of secret material in logs/diagnostic exports.
    SecretMaterialRedaction,
    /// rotation generation/overlap verification.
    RotationGenerationOverlap,
    /// secure media session verification.
    SecureMediaSession,
    /// edge termination and downstream protection verification.
    EdgeTerminationDownstreamProtection,
}

/// transport security runtime verification guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransportSecurityVerificationGuard {
    verification_class: TransportSecurityVerificationClass,
    correlation_or_startup_run_id_available: bool,
    configuration_shape_available: bool,
    secret_source_reference_available_when_checked: bool,
    listener_startup_not_used_as_peer_verification: bool,
    peer_verification_not_used_as_internal_authorization: bool,
    edge_termination_boundary_available_when_checked: bool,
    rotation_state_available_when_checked: bool,
    secure_media_session_not_inferred_from_configuration: bool,
    secret_material_redacted: bool,
}

/// transport security runtime verification guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportSecurityVerificationError {
    /// verification に required input が不足しています。
    RequiredVerificationInputMissing,
    /// listener startup と peer verification を混同しています。
    ListenerStartupConflatedWithPeerVerification,
    /// peer verification と internal service authorization を混同しています。
    PeerVerificationConflatedWithAuthorization,
    /// edge termination boundary が未宣言です。
    EdgeTerminationBoundaryMissing,
    /// rotation state が未確認です。
    RotationStateMissing,
    /// secure media session readiness を configuration から推論しています。
    SecureMediaSessionConflatedWithConfiguration,
    /// secret material redaction がありません。
    SecretRedactionMissing,
}

impl TransportSecurityVerificationGuard {
    /// runtime verification class 間の流用を禁止します。
    pub const fn try_new(
        verification_class: TransportSecurityVerificationClass,
        correlation_or_startup_run_id_available: bool,
        configuration_shape_available: bool,
        secret_source_reference_available_when_checked: bool,
        listener_startup_not_used_as_peer_verification: bool,
        peer_verification_not_used_as_internal_authorization: bool,
        edge_termination_boundary_available_when_checked: bool,
        rotation_state_available_when_checked: bool,
        secure_media_session_not_inferred_from_configuration: bool,
        secret_material_redacted: bool,
    ) -> Result<Self, TransportSecurityVerificationError> {
        if !correlation_or_startup_run_id_available || !configuration_shape_available {
            return Err(TransportSecurityVerificationError::RequiredVerificationInputMissing);
        }
        if matches!(
            verification_class,
            TransportSecurityVerificationClass::SecretSourceAvailability
                | TransportSecurityVerificationClass::SessionEstablishment
        ) && !secret_source_reference_available_when_checked
        {
            return Err(TransportSecurityVerificationError::RequiredVerificationInputMissing);
        }
        if !listener_startup_not_used_as_peer_verification {
            return Err(
                TransportSecurityVerificationError::ListenerStartupConflatedWithPeerVerification,
            );
        }
        if !peer_verification_not_used_as_internal_authorization {
            return Err(
                TransportSecurityVerificationError::PeerVerificationConflatedWithAuthorization,
            );
        }
        if matches!(
            verification_class,
            TransportSecurityVerificationClass::EdgeTerminationDownstreamProtection
        ) && !edge_termination_boundary_available_when_checked
        {
            return Err(TransportSecurityVerificationError::EdgeTerminationBoundaryMissing);
        }
        if matches!(
            verification_class,
            TransportSecurityVerificationClass::RotationGenerationOverlap
        ) && !rotation_state_available_when_checked
        {
            return Err(TransportSecurityVerificationError::RotationStateMissing);
        }
        if !secure_media_session_not_inferred_from_configuration {
            return Err(
                TransportSecurityVerificationError::SecureMediaSessionConflatedWithConfiguration,
            );
        }
        if !secret_material_redacted {
            return Err(TransportSecurityVerificationError::SecretRedactionMissing);
        }

        Ok(Self {
            verification_class,
            correlation_or_startup_run_id_available,
            configuration_shape_available,
            secret_source_reference_available_when_checked,
            listener_startup_not_used_as_peer_verification,
            peer_verification_not_used_as_internal_authorization,
            edge_termination_boundary_available_when_checked,
            rotation_state_available_when_checked,
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
    /// raw key/token/secret appears in audit/log/metric/diagnostic export.
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
