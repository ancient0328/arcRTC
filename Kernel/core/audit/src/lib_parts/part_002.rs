impl HashChainSequence {
    /// sequence number を作ります。
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// raw sequence value です。
    pub const fn value(&self) -> u64 {
        self.0
    }
}

/// v0.2 initial architecture で許可された hash algorithm です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HashAlgorithm {
    /// SHA-256 over canonical record input です。
    Sha256,
}

impl HashAlgorithm {
    /// canonical algorithm code です。
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sha256 => "sha256",
        }
    }
}

/// canonical record format/version です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CanonicalRecordFormat {
    format: &'static str,
    version: &'static str,
}

impl CanonicalRecordFormat {
    /// canonical format と version を明示します。
    pub const fn new(format: &'static str, version: &'static str) -> Self {
        Self { format, version }
    }

    /// canonical format 名です。
    pub const fn format(&self) -> &'static str {
        self.format
    }

    /// canonical format version です。
    pub const fn version(&self) -> &'static str {
        self.version
    }
}

/// canonical event payload digest です。free-text details は digest 入力に含めません。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanonicalEventPayloadDigest {
    algorithm: HashAlgorithm,
    digest: Vec<u8>,
}

impl CanonicalEventPayloadDigest {
    /// canonical event payload digest を保持します。
    pub fn new(algorithm: HashAlgorithm, digest: Vec<u8>) -> Result<Self, HashChainRecordError> {
        if digest.is_empty() {
            return Err(HashChainRecordError::EmptyDigest);
        }
        Ok(Self { algorithm, digest })
    }

    /// digest algorithm です。
    pub const fn algorithm(&self) -> HashAlgorithm {
        self.algorithm
    }

    /// digest bytes です。
    pub fn digest(&self) -> &[u8] {
        &self.digest
    }
}

/// hash-chain record hash です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordHash {
    algorithm: HashAlgorithm,
    digest: Vec<u8>,
}

impl RecordHash {
    /// record hash を保持します。
    pub fn new(algorithm: HashAlgorithm, digest: Vec<u8>) -> Result<Self, HashChainRecordError> {
        if digest.is_empty() {
            return Err(HashChainRecordError::EmptyDigest);
        }
        Ok(Self { algorithm, digest })
    }

    /// hash algorithm です。
    pub const fn algorithm(&self) -> HashAlgorithm {
        self.algorithm
    }

    /// hash bytes です。
    pub fn digest(&self) -> &[u8] {
        &self.digest
    }
}

/// previous record hash または genesis marker です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PreviousRecordHash {
    /// chain の先頭 record です。
    Genesis,
    /// 直前 record hash です。
    Previous(RecordHash),
}

/// hash-chain record contract です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashChainRecord {
    scope: HashChainScope,
    sequence: HashChainSequence,
    previous_hash: PreviousRecordHash,
    event_type: AuditEventType,
    outcome: UseCaseOutcome,
    canonical_format: CanonicalRecordFormat,
    payload_digest: CanonicalEventPayloadDigest,
    record_hash: RecordHash,
}

/// hash-chain record 生成時の入力です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashChainRecordInput {
    pub scope: HashChainScope,
    pub sequence: HashChainSequence,
    pub previous_hash: PreviousRecordHash,
    pub event_type: AuditEventType,
    pub outcome: UseCaseOutcome,
    pub canonical_format: CanonicalRecordFormat,
    pub payload_digest: CanonicalEventPayloadDigest,
    pub record_hash: RecordHash,
}

impl HashChainRecord {
    /// hash-chain record を作ります。
    pub fn new(input: HashChainRecordInput) -> Self {
        let HashChainRecordInput {
            scope,
            sequence,
            previous_hash,
            event_type,
            outcome,
            canonical_format,
            payload_digest,
            record_hash,
        } = input;

        Self {
            scope,
            sequence,
            previous_hash,
            event_type,
            outcome,
            canonical_format,
            payload_digest,
            record_hash,
        }
    }

    /// chain scope です。
    pub const fn scope(&self) -> HashChainScope {
        self.scope
    }

    /// sequence number です。
    pub const fn sequence(&self) -> HashChainSequence {
        self.sequence
    }

    /// previous hash または genesis marker です。
    pub const fn previous_hash(&self) -> &PreviousRecordHash {
        &self.previous_hash
    }

    /// audit event type です。
    pub const fn event_type(&self) -> AuditEventType {
        self.event_type
    }

    /// audit event outcome です。
    pub const fn outcome(&self) -> UseCaseOutcome {
        self.outcome
    }
}

/// hash-chain record construction error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HashChainRecordError {
    /// digest bytes が空です。
    EmptyDigest,
}

/// hash-chain verification failure class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HashChainVerificationFailure {
    /// chain gap が観測されました。
    ChainGap,
    /// sequence mismatch が観測されました。
    SequenceMismatch,
    /// previous hash mismatch が観測されました。
    PreviousHashMismatch,
    /// unknown algorithm が観測されました。
    UnknownAlgorithm,
    /// canonical serialization failure が観測されました。
    CanonicalSerializationFailed,
    /// canonicalization mismatch が観測されました。
    CanonicalizationMismatch,
}
