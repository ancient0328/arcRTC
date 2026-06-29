impl EdgeProxyHeaderSourceGuard {
    /// forwarded header は名前だけでは信用しないことを検査します。
    pub const fn try_new(
        edge_class: EdgeProxyClass,
        metadata_class: TrustedMetadataClass,
        immediate_upstream_trusted_by_policy: bool,
        deterministic_precedence_declared_when_sources_conflict: bool,
        raw_metadata_not_used_as_core_identity: bool,
        mapped_only_to_typed_policy_input: bool,
        proxy_request_id_not_used_as_correlation_id: bool,
    ) -> Result<Self, EdgeProxyHeaderSourceError> {
        if !edge_class.admits_metadata_class(metadata_class) {
            return Err(EdgeProxyHeaderSourceError::MetadataClassNotAdmittedForEdgeClass);
        }
        if !immediate_upstream_trusted_by_policy {
            return Err(EdgeProxyHeaderSourceError::ImmediateUpstreamUntrusted);
        }
        if !deterministic_precedence_declared_when_sources_conflict {
            return Err(EdgeProxyHeaderSourceError::DeterministicPrecedenceMissing);
        }
        if !raw_metadata_not_used_as_core_identity
            || !metadata_class.raw_value_must_not_be_core_identity()
        {
            return Err(EdgeProxyHeaderSourceError::RawMetadataUsedAsCoreIdentity);
        }
        if !mapped_only_to_typed_policy_input {
            return Err(EdgeProxyHeaderSourceError::TypedPolicyInputMissing);
        }
        if !proxy_request_id_not_used_as_correlation_id {
            return Err(EdgeProxyHeaderSourceError::ProxyRequestIdReplacesCorrelationId);
        }

        Ok(Self {
            edge_class,
            metadata_class,
            immediate_upstream_trusted_by_policy,
            deterministic_precedence_declared_when_sources_conflict,
            raw_metadata_not_used_as_core_identity,
            mapped_only_to_typed_policy_input,
            proxy_request_id_not_used_as_correlation_id,
        })
    }
}

/// edge TLS termination class の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeTlsTerminationClass {
    /// no TLS termination at edge.
    NoEdgeTermination,
    /// TLS terminates at trusted edge and backend remains protected.
    TrustedEdgeTerminatesWithProtectedBackend,
    /// TLS termination relation is not admitted.
    TerminationNotAdmitted,
}

/// TLS termination relation guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeTlsTerminationGuard {
    termination_class: EdgeTlsTerminationClass,
    backend_transport_security_requirement_declared: bool,
    trusted_edge_identity_declared: bool,
    certificate_or_secret_reference_class_declared: bool,
    audit_evidence_relation_declared: bool,
    missing_downstream_protection_failure_mapping_declared: bool,
    edge_tls_not_used_as_secure_media_proof: bool,
}

/// TLS termination relation の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeTlsTerminationError {
    /// TLS termination relation が許可されていません。
    TerminationClassNotAdmitted,
    /// backend transport security requirement がありません。
    BackendTransportSecurityRequirementMissing,
    /// trusted edge identity がありません。
    TrustedEdgeIdentityMissing,
    /// certificate/secret reference class がありません。
    CertificateSecretReferenceClassMissing,
    /// audit evidence relation がありません。
    AuditEvidenceRelationMissing,
    /// missing downstream protection の failure mapping がありません。
    MissingDownstreamProtectionFailureMappingMissing,
    /// edge TLS を secure media proof として扱っています。
    EdgeTlsUsedAsSecureMediaProof,
}

impl EdgeTlsTerminationGuard {
    /// edge TLS termination と backend protection を別物として検査します。
    pub const fn try_new(
        termination_class: EdgeTlsTerminationClass,
        backend_transport_security_requirement_declared: bool,
        trusted_edge_identity_declared: bool,
        certificate_or_secret_reference_class_declared: bool,
        audit_evidence_relation_declared: bool,
        missing_downstream_protection_failure_mapping_declared: bool,
        edge_tls_not_used_as_secure_media_proof: bool,
    ) -> Result<Self, EdgeTlsTerminationError> {
        if matches!(
            termination_class,
            EdgeTlsTerminationClass::TerminationNotAdmitted
        ) {
            return Err(EdgeTlsTerminationError::TerminationClassNotAdmitted);
        }
        if !backend_transport_security_requirement_declared {
            return Err(EdgeTlsTerminationError::BackendTransportSecurityRequirementMissing);
        }
        if !trusted_edge_identity_declared {
            return Err(EdgeTlsTerminationError::TrustedEdgeIdentityMissing);
        }
        if !certificate_or_secret_reference_class_declared {
            return Err(EdgeTlsTerminationError::CertificateSecretReferenceClassMissing);
        }
        if !audit_evidence_relation_declared {
            return Err(EdgeTlsTerminationError::AuditEvidenceRelationMissing);
        }
        if !missing_downstream_protection_failure_mapping_declared {
            return Err(EdgeTlsTerminationError::MissingDownstreamProtectionFailureMappingMissing);
        }
        if !edge_tls_not_used_as_secure_media_proof {
            return Err(EdgeTlsTerminationError::EdgeTlsUsedAsSecureMediaProof);
        }

        Ok(Self {
            termination_class,
            backend_transport_security_requirement_declared,
            trusted_edge_identity_declared,
            certificate_or_secret_reference_class_declared,
            audit_evidence_relation_declared,
            missing_downstream_protection_failure_mapping_declared,
            edge_tls_not_used_as_secure_media_proof,
        })
    }
}

/// edge/proxy trust evidence guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeProxyTrustEvidenceGuard {
    edge_class_declared: bool,
    topology_class_declared: bool,
    trusted_upstream_scope_declared: bool,
    accepted_metadata_classes_declared: bool,
    header_precedence_declared: bool,
    hop_count_declared: bool,
    tls_termination_relation_declared: bool,
    origin_host_policy_declared: bool,
    client_address_use_limit_declared: bool,
    command_or_procedure_declared: bool,
    working_directory_declared: bool,
    rerun_condition_declared: bool,
    diagnostic_snippet_not_used_without_required_fields: bool,
}

/// edge/proxy trust evidence の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeProxyTrustEvidenceError {
    /// edge class がありません。
    EdgeClassMissing,
    /// topology class がありません。
    TopologyClassMissing,
    /// trusted upstream scope がありません。
    TrustedUpstreamScopeMissing,
    /// accepted metadata classes がありません。
    AcceptedMetadataClassesMissing,
    /// header precedence がありません。
    HeaderPrecedenceMissing,
    /// hop count がありません。
    HopCountMissing,
    /// TLS termination relation がありません。
    TlsTerminationRelationMissing,
    /// origin/host policy がありません。
    OriginHostPolicyMissing,
    /// client address use limit がありません。
    ClientAddressUseLimitMissing,
    /// command/procedure がありません。
    CommandProcedureMissing,
    /// working directory がありません。
    WorkingDirectoryMissing,
    /// rerun condition がありません。
    RerunConditionMissing,
    /// diagnostic snippet だけを evidence として使っています。
    DiagnosticSnippetUsedAsEvidence,
}

impl EdgeProxyTrustEvidenceGuard {
    /// edge/proxy trust evidence の採用条件を検査します。
    pub const fn try_new(
        edge_class_declared: bool,
        topology_class_declared: bool,
        trusted_upstream_scope_declared: bool,
        accepted_metadata_classes_declared: bool,
        header_precedence_declared: bool,
        hop_count_declared: bool,
        tls_termination_relation_declared: bool,
        origin_host_policy_declared: bool,
        client_address_use_limit_declared: bool,
        command_or_procedure_declared: bool,
        working_directory_declared: bool,
        rerun_condition_declared: bool,
        diagnostic_snippet_not_used_without_required_fields: bool,
    ) -> Result<Self, EdgeProxyTrustEvidenceError> {
        if !edge_class_declared {
            return Err(EdgeProxyTrustEvidenceError::EdgeClassMissing);
        }
        if !topology_class_declared {
            return Err(EdgeProxyTrustEvidenceError::TopologyClassMissing);
        }
        if !trusted_upstream_scope_declared {
            return Err(EdgeProxyTrustEvidenceError::TrustedUpstreamScopeMissing);
        }
        if !accepted_metadata_classes_declared {
            return Err(EdgeProxyTrustEvidenceError::AcceptedMetadataClassesMissing);
        }
        if !header_precedence_declared {
            return Err(EdgeProxyTrustEvidenceError::HeaderPrecedenceMissing);
        }
        if !hop_count_declared {
            return Err(EdgeProxyTrustEvidenceError::HopCountMissing);
        }
        if !tls_termination_relation_declared {
            return Err(EdgeProxyTrustEvidenceError::TlsTerminationRelationMissing);
        }
        if !origin_host_policy_declared {
            return Err(EdgeProxyTrustEvidenceError::OriginHostPolicyMissing);
        }
        if !client_address_use_limit_declared {
            return Err(EdgeProxyTrustEvidenceError::ClientAddressUseLimitMissing);
        }
        if !command_or_procedure_declared {
            return Err(EdgeProxyTrustEvidenceError::CommandProcedureMissing);
        }
        if !working_directory_declared {
            return Err(EdgeProxyTrustEvidenceError::WorkingDirectoryMissing);
        }
        if !rerun_condition_declared {
            return Err(EdgeProxyTrustEvidenceError::RerunConditionMissing);
        }
        if !diagnostic_snippet_not_used_without_required_fields {
            return Err(EdgeProxyTrustEvidenceError::DiagnosticSnippetUsedAsEvidence);
        }

        Ok(Self {
            edge_class_declared,
            topology_class_declared,
            trusted_upstream_scope_declared,
            accepted_metadata_classes_declared,
            header_precedence_declared,
            hop_count_declared,
            tls_termination_relation_declared,
            origin_host_policy_declared,
            client_address_use_limit_declared,
            command_or_procedure_declared,
            working_directory_declared,
            rerun_condition_declared,
            diagnostic_snippet_not_used_without_required_fields,
        })
    }
}

/// edge/proxy trust failure mapping の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeProxyTrustFailureKind {
    /// edge/proxy class is not admitted.
    EdgeProxyNotAdmitted,
    /// forwarded/trusted header is not admitted or not trustworthy.
    ForwardedHeaderUntrusted,
    /// forwarded header chain exceeds accepted hop count or conflicts.
    ForwardedHeaderChainInvalid,
    /// origin or host is not allowed by policy.
    OriginHostNotAllowed,
    /// client address cannot be trusted for target decision.
    ClientAddressUntrusted,
    /// TLS termination/downstream security relation is invalid.
    TlsTerminationBoundaryInvalid,
    /// public/internal route is confused by edge/proxy mapping.
    PublicInternalRouteConfusion,
}

impl EdgeProxyTrustFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::EdgeProxyNotAdmitted => "edge_proxy_not_admitted",
            Self::ForwardedHeaderUntrusted => "forwarded_header_untrusted",
            Self::ForwardedHeaderChainInvalid => "forwarded_header_chain_invalid",
            Self::OriginHostNotAllowed => "origin_host_not_allowed",
            Self::ClientAddressUntrusted => "client_address_untrusted",
            Self::TlsTerminationBoundaryInvalid => "tls_termination_boundary_invalid",
            Self::PublicInternalRouteConfusion => "public_internal_route_confusion",
        }
    }
}

/// edge/proxy trust failure です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeProxyTrustFailure {
    kind: EdgeProxyTrustFailureKind,
    reason: CatalogedReasonRef,
}

impl EdgeProxyTrustFailure {
    /// edge/proxy trust failure を cataloged reason に接続します。
    pub fn from_kind(kind: EdgeProxyTrustFailureKind) -> Self {
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("edge/proxy trust reason code must be registered");
        Self { kind, reason }
    }
}

/// edge/proxy trust 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedEdgeProxyTrustBehavior {
    /// forwarded header is trusted because it has a standard name.
    ForwardedHeaderTrustedByStandardName,
    /// client IP, host, origin, or SNI becomes core identity.
    RawEdgeMetadataBecomesCoreIdentity,
    /// public/internal route separation depends only on proxy route naming.
    RouteSeparationDependsOnlyOnProxyNaming,
    /// edge TLS termination is treated as backend secure transport or secure media proof.
    EdgeTlsTerminationTreatedAsBackendOrMediaProof,
    /// rate/quota/admission policy uses raw forwarded header without trust policy.
    RawForwardedHeaderUsedForPolicyWithoutTrust,
    /// proxy request ID replaces core CorrelationId.
    ProxyRequestIdReplacesCorrelationId,
    /// service mesh identity becomes application authorization context by default.
    MeshIdentityBecomesApplicationAuthorizationByDefault,
}
