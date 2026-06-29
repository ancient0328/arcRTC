# 第08章 reference runtime state output

状態: SSOT 統合版
日付: 2026-06-28 JST

## 目的

本章は、arcRTC v0.2 implementations 領域における reference implementation の phase / fixture payload（全 phase と payload）、state mutation 規則（全状態遷移）、reference-local API 形状と state 型、output boundary（product が消費できる 3-type allow-list）、runtime lifecycle（全状態）、command evidence runner（実行と evidence 生成手順）を、再現実装可能な粒度で固定します。本章は完全自己完結であり、他文書・実コードを参照せずに理解できます。同一仕様書内の他章番号のみ参照します。

reference state は local deterministic behavior projection であり、Kernel state authority、database state、production state、readiness evidence ではありません。reference fixture は deterministic local input であり、production credential、real SDP、real ICE candidate、real packet payload、readiness evidence ではありません。

---

## 1. Signaling local phase と fixture payload

### 1.1 Signaling local phase enum

`reference-implementation/signaling/src/state.rs` は次の local phase enum を持ちます。

| Enum | Variants | 規則 |
|---|---|---|
| `ReferenceRoomPhase` | `Open`, `Closed` | absent room は map key 不在で表現する |
| `ReferenceParticipantPhase` | `Joined`, `Left` | absent participant は map key 不在で表現する |

`JoinRoom` success は `ReferenceRoomPhase::Open` と `ReferenceParticipantPhase::Joined` を作ります。`LeaveRoom` success は participant を削除せず `ReferenceParticipantPhase::Left` にします。`Closed` room への join は `STATE_BOUNDARY_VIOLATION` で拒否します。

### 1.2 Signaling fixture 型

`reference-implementation/signaling/src/fixture_identity.rs` は次を持ちます。

| Type | 必須フィールド | 値規則 |
|---|---|---|
| `FixtureIdentity` | `identity_id: String`, `participant_id: ParticipantId`, `room_id: RoomId` | `identity_id` は `fixture-identity-{n}` |
| `FixtureSessionDescription` | `session_id: SessionId`, `fixture_sdp_id: String`, `direction: FixtureSessionDescriptionDirection` | `fixture_sdp_id` は `fixture-sdp-{n}` |
| `FixtureSessionDescriptionDirection` | enum `Offer`, `Answer` | offer / answer fixture 分類のみ |
| `FixtureIceCandidate` | `session_id: SessionId`, `fixture_candidate_id: String` | `fixture_candidate_id` は `fixture-ice-candidate-{n}` |

real SDP text と real ICE candidate body は fixture 型に格納しません（MUST NOT）。

### 1.3 Signaling payload variants

`ReferenceSignalingPayload` variants は次に固定します。

- `JoinRoom`
- `LeaveRoom`
- `SendOffer { session: FixtureSessionDescription }`
- `SendAnswer { session: FixtureSessionDescription }`
- `SendIceCandidate { candidate: FixtureIceCandidate }`
- `RequestTurnCredential`
- `AcknowledgeForward`

`SendOffer` は `FixtureSessionDescriptionDirection::Offer` を要求します。`SendAnswer` は `FixtureSessionDescriptionDirection::Answer` を要求します。direction mismatch は `FIXTURE_IDENTITY_INVALID` です。

### 1.4 TURN fixture 型

`reference-implementation/turn/src/fixture_credential.rs` は次に固定します。

| Type / Function | 必須形状 | 規則 |
|---|---|---|
| `FixtureTurnCredential` | `credential_ref: CredentialRef`, `allocation_id: AllocationId`, `requested_lifetime: TurnRequestedLifetimeSeconds` | deterministic fixture credential のみ |
| `validate_fixture_turn_credential` | `(&FixtureTurnCredential) -> Result<(), ReferenceTurnError>` | missing / invalid fixture identity を拒否する |

`FixtureTurnCredential` は raw secret、password、private key、provider token を含みません（MUST NOT）。

### 1.5 SFU fixture 型

`reference-implementation/sfu/src/fixture_route_auth.rs` は次に固定します。

| Type / Function | 必須形状 | 規則 |
|---|---|---|
| `FixtureRouteAdmission` | `route_id: RouteId`, `stream_id: StreamId`, `endpoint_id: EndpointId`, `allowed: bool` | deterministic route admission のみ |
| `authorize_reference_route` | `(&FixtureRouteAdmission) -> Result<(), ReferenceSfuError>` | `allowed == false` は `FIXTURE_IDENTITY_INVALID` で拒否する |

### 1.6 SFU local phase enum

`reference-implementation/sfu/src/state.rs` は次の local phase enum を所有します。

| Enum | Variants | 規則 |
|---|---|---|
| `SfuSessionState` | `Open`, `Draining`, `Closed` | absent session は map key 不在で表現する |
| `SfuEndpointState` | `Observed`, `AdmissionPending`, `Admitted`, `Degraded`, `Draining`, `Removed`, `Rejected` | absent endpoint は map key 不在で表現する |
| `SfuRouteState` | `Candidate`, `Selected`, `Delayed`, `SuppressedByBackpressure`, `SuppressedByQuality`, `Degraded`, `Dropped`, `Closed` | absent route は map key 不在で表現する |

`ReferenceSfuAction` は SFU reference state を mutate できる唯一の local input enum です。これらの local SFU state enum は frozen Kernel state variant 名を反映し、implementation-only variant を導入してはなりません（MUST NOT）。

### 1.7 cross-plane fixture 規則

composition binding は reference state に既に存在する concrete Kernel identity 値を使用します。composition は production identity、production credential、provider state を合成してはなりません（MUST NOT）。

| Binding | 必須 source |
|---|---|
| signaling to TURN | joined reference signaling participant と active reference TURN allocation |
| signaling to SFU | joined reference signaling participant と existing reference SFU route |

### 1.8 崩壊条件

- real SDP body、real ICE candidate body、raw TURN secret、real provider token を fixture に保存する。
- absent state と rejected state を同じ enum variant で表現する。
- `LeaveRoom` が participant record を消して evidence trace を失う。
- fixture validation failure を `UNKNOWN` または success として扱う。
- fixture identity を product identity source of truth として扱う。

---

## 2. state mutation 規則（全状態遷移）

reference state は in-memory state mutation です。本節は map key、duplicate handling、reject branch を固定します。

### 2.1 共有 state 規則（map key）

すべての reference state map は Kernel identity 値由来の canonical key 文字列を使用します。

| ID type | key 規則 |
|---|---|
| `RoomId` | `room_id.as_str().to_owned()` |
| `ParticipantId` | `participant_id.as_str().to_owned()` |
| `AllocationId` | `allocation_id.as_str().to_owned()` |
| `PermissionId` | `permission_id.as_str().to_owned()` |
| `ChannelBindId` | `channel_bind_id.as_str().to_owned()` |
| `SessionId` | `session_id.as_str().to_owned()` |
| `EndpointId` | `endpoint_id.as_str().to_owned()` |
| `RouteId` | `route_id.as_str().to_owned()` |

いかなる state map key も provider id、random id、wall-clock time、memory address、raw packet bytes、fixture display text を authority として使用してはなりません（MUST NOT）。

### 2.2 Signaling mutation table

| Command | Pre-state | Mutation | Outcome | Reject branch |
|---|---|---|---|---|
| `JoinRoom` | room absent | `ReferenceRoomState { phase: Open }` を insert; participant key を room set に insert; participant state `Joined` を insert | `SignalingEventKind::Joined` | none |
| `JoinRoom` | room `Open`, participant absent | participant key を room set に insert; participant state `Joined` を insert | `SignalingEventKind::Joined` | none |
| `JoinRoom` | 同一 room の participant `Left` | room を保持; participant を `Joined` に; room set が participant key を含むことを保証 | `SignalingEventKind::Joined` | none |
| `JoinRoom` | 同一 room の participant `Joined` | mutation なし | `SignalingEventKind::Joined` | none |
| `JoinRoom` | room `Closed` または別 room で joined の participant | mutation なし | none | `ReferenceSignalingError::StateBoundaryViolation` |
| `LeaveRoom` | target room で participant `Joined` | participant を `Left` に; evidence trace のため room set の participant key を保持 | `SignalingEventKind::Left` | none |
| `LeaveRoom` | participant absent / left / wrong room | mutation なし | none | `ReferenceSignalingError::StateBoundaryViolation` |
| `SendOffer` | room で participant `Joined` かつ fixture direction `Offer` | state mutation なし | `SignalingEventKind::OfferReceived` | fixture mismatch: `InvalidFixtureIdentity`; state mismatch: `StateBoundaryViolation` |
| `SendAnswer` | room で participant `Joined` かつ fixture direction `Answer` | state mutation なし | `SignalingEventKind::AnswerReceived` | fixture mismatch: `InvalidFixtureIdentity`; state mismatch: `StateBoundaryViolation` |
| `SendIceCandidate` | room で participant `Joined` | state mutation なし | `SignalingEventKind::IceCandidateReceived` | fixture mismatch: `InvalidFixtureIdentity`; state mismatch: `StateBoundaryViolation` |
| `RequestTurnCredential` | room で participant `Joined` | state mutation なし | `SignalingEventKind::TurnCredentialAvailable` | state mismatch: `StateBoundaryViolation` |
| `AcknowledgeForward` | room で participant `Joined` | state mutation なし | `SignalingEventKind::Joined` | state mismatch: `StateBoundaryViolation` |

### 2.3 TURN mutation table

| Command | Pre-state | Mutation | Outcome | Reject branch |
|---|---|---|---|---|
| `Allocate` | allocation absent かつ credential fixture valid | allocation `AllocationState::Active` を insert; credential ref を格納 | `TurnDecisionKind::Allocation` | invalid credential: `InvalidFixtureCredential` |
| `Allocate` | allocation active | mutation なし | `TurnDecisionKind::Allocation` | none |
| `Refresh` | allocation active | allocation active を保持 | `TurnDecisionKind::Refresh` | missing allocation: `StateBoundaryViolation` |
| `CreatePermission` | allocation active, permission absent, peer address present | permission `PermissionState::Active` を insert | `TurnDecisionKind::Permission` | missing allocation / peer: `StateBoundaryViolation` |
| `CreatePermission` | 同一 allocation / peer の permission active | mutation なし | `TurnDecisionKind::Permission` | none |
| `ChannelBind` | permission active かつ channel bind absent | channel bind `ChannelBindState::Active` を insert | `TurnDecisionKind::ChannelBind` | missing permission / bind id: `StateBoundaryViolation` |
| `ChannelBind` | channel bind active | mutation なし | `TurnDecisionKind::ChannelBind` | none |
| `RelayData` | allocation active, permission active, packet id present | state mutation なし | `TurnDecisionKind::Relay` | missing allocation / permission / packet: `StateBoundaryViolation` |

TURN validation は次を拒否します: allocation key が absent な permission、permission key が absent な channel bind、command path で peer address が absent な permission。

### 2.4 SFU mutation table

SFU mutation は `ReferenceSfuAction` によって駆動され、Kernel `SfuReferenceSet` の private field 読み取りによらないこと。outcome `SfuDecisionKind` 値は frozen Kernel 値です（第3節の mapping 表参照）。`ReferenceSfuAction` の field は typed local input であり、typed action field は mutation table 内で missing として扱いません。下記 reject branch は state absence、state mismatch、disallowed phase を指し、absent Rust field を指しません。

| Reference action | Pre-state | Mutation | Outcome | Reject branch |
|---|---|---|---|---|
| `AdmitParticipant` | session absent かつ endpoint absent | session `SfuSessionState::Open` を insert; endpoint `SfuEndpointState::Admitted` を session に bind して insert | `SfuDecisionKind::ParticipantAdmission` | none |
| `AdmitParticipant` | session open かつ endpoint absent / observed / admission pending / degraded / draining | endpoint を session に bind して `SfuEndpointState::Admitted` に | `SfuDecisionKind::ParticipantAdmission` | none |
| `AdmitParticipant` | session open かつ同一 session の endpoint admitted | mutation なし | `SfuDecisionKind::ParticipantAdmission` | none |
| `AdmitParticipant` | session closed / draining または endpoint rejected / removed / 別 session bind | mutation なし | none | `ReferenceSfuError::StateBoundaryViolation` |
| `RejectParticipant` | session absent かつ endpoint absent | session `SfuSessionState::Open` を insert; endpoint `SfuEndpointState::Rejected` を session に bind して insert | `SfuDecisionKind::ParticipantAdmission` | none |
| `RejectParticipant` | session open かつ endpoint absent / observed / admission pending | endpoint を session に bind して `SfuEndpointState::Rejected` に | `SfuDecisionKind::ParticipantAdmission` | none |
| `RejectParticipant` | session open かつ同一 session の endpoint rejected | mutation なし | `SfuDecisionKind::ParticipantAdmission` | none |
| `RejectParticipant` | session closed / draining, endpoint admitted / degraded / draining / removed, または別 session bind | mutation なし | none | `ReferenceSfuError::StateBoundaryViolation` |
| `PublishStream` | session open かつ同一 session の endpoint admitted | state mutation なし; publication は outcome のみで表現 | `SfuDecisionKind::Publication` | none |
| `PublishStream` | session absent / closed / draining, endpoint absent / not admitted, または別 session bind | mutation なし | none | `ReferenceSfuError::StateBoundaryViolation` |
| `SubscribeRoute` | session open, 同一 session の endpoint admitted, route absent | action が運ぶ route id / session id / endpoint id / stream id で route `SfuRouteState::Candidate` を insert | `SfuDecisionKind::Subscription` | none |
| `SubscribeRoute` | session open, 同一 session の endpoint admitted, 同一 session / endpoint / stream の route candidate | mutation なし | `SfuDecisionKind::Subscription` | none |
| `SubscribeRoute` | session absent / closed / draining, endpoint absent / not admitted, 別 session bind, route closed / dropped, または route の session / endpoint / stream mismatch | mutation なし | none | `ReferenceSfuError::StateBoundaryViolation` |
| `SelectRoute` | 同一 session / endpoint / stream の route candidate または selected | route `SfuRouteState::Selected` に | `SfuDecisionKind::RouteSelection` | none |
| `SelectRoute` | route absent / dropped / closed, または route の session / endpoint / stream mismatch | mutation なし | none | `ReferenceSfuError::StateBoundaryViolation` |
| `SuppressForwarding { source: Backpressure }` | route selected | route `SfuRouteState::SuppressedByBackpressure` に | `SfuDecisionKind::Forwarding` | none |
| `SuppressForwarding { source: Quality }` | route selected | route `SfuRouteState::SuppressedByQuality` に | `SfuDecisionKind::Forwarding` | none |
| `SuppressForwarding` | route absent / not selected / dropped / closed | mutation なし | none | `ReferenceSfuError::StateBoundaryViolation` |
| `DropForwarding` | route candidate / selected / delayed / suppressed by backpressure / suppressed by quality / degraded | route `SfuRouteState::Dropped` に | `SfuDecisionKind::BackpressureAction` | none |
| `DropForwarding` | route absent / closed | mutation なし | none | `ReferenceSfuError::StateBoundaryViolation` |
| `CloseSession` | session open / draining | session `SfuSessionState::Closed` に; その session の endpoints を `SfuEndpointState::Removed` に; その session の routes を `SfuRouteState::Closed` に | `SfuDecisionKind::DegradationRecovery` | none |
| `CloseSession` | session absent / closed | mutation なし | none | `ReferenceSfuError::StateBoundaryViolation` |

SFU state は raw packet bytes または borrowed packet slice を格納してはなりません（MUST NOT）。SFU state は stream map を持ちません。`stream_id` は route validation のため `ReferenceRouteState` にのみ保持し、独立した stream authority store にしてはなりません。

### 2.5 Composition mutation table

| Function | Pre-state | Mutation | Reject branch |
|---|---|---|---|
| `bind_signaling_to_turn` | room exists/open かつ allocation active | binding key `correlation_id + room_id + allocation_id` を insert | missing room / allocation: `ReferenceCompositionError::StateBoundaryViolation` |
| `bind_signaling_to_sfu` | room exists/open かつ route exists | binding key `correlation_id + room_id + session_id + route_id` を insert | missing room / route: `ReferenceCompositionError::StateBoundaryViolation` |
| `validate_reference_composition_state` | すべての binding reference が存在 | mutation なし | orphan binding: `ReferenceCompositionError::StateBoundaryViolation` |

composition binding は reference のみを記録します。composition は plane state を新しい authority store に copy してはなりません（MUST NOT）。

### 2.6 崩壊条件

- absent state と rejected state を同じ mutation result で表現する。
- state map key が random / provider / wall-clock / memory / raw payload data を使用する。
- state map key が Kernel `as_str()` 値の代わりに `Debug` 表現を使用する。
- `LeaveRoom` が participant evidence trace を削除する。
- SFU state が raw packet bytes または borrowed packet slice を格納する。
- composition が product authority state を作る。

---

## 3. reference-local API と state 型

本節は reference-local API、state field、state transition、Kernel contract builder mapping を固定します。この章は Kernel semantics を再定義しません。implementations-local 型は Kernel public contract を作るための入力、projection、evidence に限定します。

### 3.1 共有 API 規則

reference package の reference-local API は次の形に固定します。

| API class | 必須命名 | Return boundary |
|---|---|---|
| input constructor | `Reference{Plane}Input::new(...)` | implementations-local input |
| Kernel contract builder | `build_kernel_{plane}_command(...)` または `build_kernel_{plane}_item(...)` | Kernel public contract type |
| state application | `apply_reference_{plane}(...)` | implementations-local outcome |
| event / evidence projection | `project_reference_{plane}_event(...)` | Kernel event または evidence record |
| validation | `validate_reference_{plane}_state(...)` | implementations-local validation result |

payload accessor を公開しない Kernel contract object を local source of truth として扱ってはなりません（MUST NOT）。reference behavior application の source は元の implementations-local input のままです。

### 3.2 product input allow-list 境界

product implementation が消費できる reference 型は `arcrtc-reference-output` が所有する `ReferenceSignalingOutcome`、`ReferenceTurnOutcome`、`ReferenceSfuOutcome` に限定します。

`Reference*State`、`ReferenceSfuAction`、fixture、local auth、runtime、composition state、`apply_reference_*`、`validate_reference_*` は reference behavior / reference tests / reference composition 用であり、product plane package の input source ではありません。

### 3.3 Signaling API

**必須型**

| Type | File | 必須フィールド |
|---|---|---|
| `ReferenceSignalingCommandInput` | `reference-implementation/signaling/src/kernel_contract.rs` | `correlation_id: CorrelationId`, `room_id: RoomId`, `participant_id: Option<ParticipantId>`, `kind: SignalingCommandKind`, `payload: ReferenceSignalingPayload` |
| `ReferenceSignalingPayload` | `reference-implementation/signaling/src/kernel_contract.rs` | 第1.3節の enum variants |
| `ReferenceSignalingState` | `reference-implementation/signaling/src/state.rs` | `rooms: BTreeMap<String, ReferenceRoomState>`, `participants: BTreeMap<String, ReferenceParticipantState>` |
| `ReferenceRoomState` | `reference-implementation/signaling/src/state.rs` | `room_id: RoomId`, `phase: ReferenceRoomPhase`, `participant_ids: BTreeSet<String>` |
| `ReferenceParticipantState` | `reference-implementation/signaling/src/state.rs` | `participant_id: ParticipantId`, `room_id: RoomId`, `phase: ReferenceParticipantPhase` |
| `ReferenceSignalingOutcome` | `reference-implementation/output/src/signaling.rs` | `correlation_id: CorrelationId`, `kind: SignalingEventKind`, `room_id: RoomId`, `participant_id: Option<ParticipantId>`, `implementation_reason: ImplementationEvidenceReason` |

**必須関数**

| Function | 必須シグネチャ形状 | 規則 |
|---|---|---|
| `build_kernel_signaling_command` | `(&ReferenceSignalingCommandInput) -> Result<SignalingCommand<ReferenceSignalingPayload>, ReferenceSignalingError>` | `TargetSurface::Signaling` で `CommandEnvelope<SignalingSubject>` を構築 |
| `apply_reference_signaling` | `(&mut ReferenceSignalingState, &ReferenceSignalingCommandInput) -> Result<ReferenceSignalingOutcome, ReferenceSignalingError>` | Kernel command private payload ではなく local input を使う |
| `project_reference_signaling_event` | `(&ReferenceSignalingOutcome, ReferenceSignalingPayload) -> SignalingEvent<ReferenceSignalingPayload>` | Kernel event projection を emit |
| `validate_reference_signaling_state` | `(&ReferenceSignalingState) -> Result<(), ReferenceSignalingError>` | impossible room / participant relation を拒否 |

**state transition**

| Command kind | 必須 precondition | Success effect | Reject reason |
|---|---|---|---|
| `JoinRoom` | room absent/open かつ participant absent/left | room open, participant joined | `STATE_BOUNDARY_VIOLATION` |
| `JoinRoom` | 同一 room で participant already joined | idempotent success; mutation なし | none |
| `LeaveRoom` | room で participant joined | participant left | `STATE_BOUNDARY_VIOLATION` |
| `SendOffer` | participant joined | event `OfferReceived` | `FIXTURE_IDENTITY_INVALID` または `STATE_BOUNDARY_VIOLATION` |
| `SendAnswer` | participant joined | event `AnswerReceived` | `FIXTURE_IDENTITY_INVALID` または `STATE_BOUNDARY_VIOLATION` |
| `SendIceCandidate` | participant joined | event `IceCandidateReceived` | `FIXTURE_IDENTITY_INVALID` または `STATE_BOUNDARY_VIOLATION` |
| `RequestTurnCredential` | participant joined | event `TurnCredentialAvailable` | `FIXTURE_IDENTITY_INVALID` または `STATE_BOUNDARY_VIOLATION` |
| `AcknowledgeForward` | participant joined | event `Joined` または forwarding acknowledgement | `STATE_BOUNDARY_VIOLATION` |

### 3.4 TURN API

**必須型**

| Type | File | 必須フィールド |
|---|---|---|
| `ReferenceTurnCommandInput` | `reference-implementation/turn/src/kernel_contract.rs` | `kind: TurnCommandKind`, `transaction_id: TurnTransactionId`, `references: TurnReferenceSet`, `allocation_id: Option<AllocationId>`, `permission_id: Option<PermissionId>`, `channel_bind_id: Option<ChannelBindId>`, `credential_ref: Option<CredentialRef>`, `peer_address: Option<CorePeerAddress>`, `requested_lifetime: Option<TurnRequestedLifetimeSeconds>`, `relay_packet_id: Option<PacketId>` |
| `ReferenceTurnState` | `reference-implementation/turn/src/state.rs` | `allocations: BTreeMap<String, ReferenceAllocationState>`, `permissions: BTreeMap<String, ReferencePermissionState>`, `channel_binds: BTreeMap<String, ReferenceChannelBindState>` |
| `ReferenceAllocationState` | `reference-implementation/turn/src/state.rs` | `allocation_id: AllocationId`, `phase: AllocationState`, `credential_ref: Option<CredentialRef>` |
| `ReferencePermissionState` | `reference-implementation/turn/src/state.rs` | `permission_id: PermissionId`, `allocation_id: AllocationId`, `peer_address: CorePeerAddress`, `phase: PermissionState` |
| `ReferenceChannelBindState` | `reference-implementation/turn/src/state.rs` | `channel_bind_id: ChannelBindId`, `permission_id: PermissionId`, `phase: ChannelBindState` |
| `ReferenceTurnOutcome` | `reference-implementation/output/src/turn.rs` | `kind: TurnDecisionKind`, `implementation_reason: ImplementationEvidenceReason` |

**必須関数**

| Function | 必須シグネチャ形状 | 規則 |
|---|---|---|
| `build_kernel_turn_command` | `(&ReferenceTurnCommandInput) -> Result<TurnCommand, ReferenceTurnError>` | shape validation を `TurnCommand::try_new` に委譲 |
| `apply_reference_turn` | `(&mut ReferenceTurnState, &ReferenceTurnCommandInput) -> Result<ReferenceTurnOutcome, ReferenceTurnError>` | in-memory state のみ更新; frozen Kernel `TurnReferenceSet` が reference accessor を公開しないため `ReferenceTurnCommandInput` の local reference field が mutation source |
| `validate_reference_turn_state` | `(&ReferenceTurnState) -> Result<(), ReferenceTurnError>` | orphan permission / channel bind を拒否 |

**state transition**

| Command kind | 必須 precondition | Success effect | Reject reason |
|---|---|---|---|
| `Allocate` | credential fixture valid | allocation active | `FIXTURE_IDENTITY_INVALID` |
| `Refresh` | allocation active | lifetime projection refreshed | `STATE_BOUNDARY_VIOLATION` |
| `CreatePermission` | allocation active かつ peer address present | permission active | `STATE_BOUNDARY_VIOLATION` |
| `ChannelBind` | permission active かつ channel bind id present | channel bind active | `STATE_BOUNDARY_VIOLATION` |
| `RelayData` | allocation と permission active, packet id present | relay allowed outcome | `STATE_BOUNDARY_VIOLATION` |

### 3.5 SFU API

**必須型**

| Type | File | 必須フィールド |
|---|---|---|
| `ReferenceSfuContractInput<Payload>` | `reference-implementation/sfu/src/kernel_contract.rs` | `model_kind: SfuModelKind`, `references: SfuReferenceSet`, `payload: Payload` |
| `ReferenceSfuAction` | `reference-implementation/sfu/src/state.rs` | 下記 enum variants |
| `ReferenceSfuSuppressionSource` | `reference-implementation/sfu/src/state.rs` | enum variants `Backpressure`, `Quality` |
| `ReferenceSfuState` | `reference-implementation/sfu/src/state.rs` | `sessions: BTreeMap<String, ReferenceSfuSessionState>`, `endpoints: BTreeMap<String, ReferenceEndpointState>`, `routes: BTreeMap<String, ReferenceRouteState>` |
| `ReferenceSfuSessionState` | `reference-implementation/sfu/src/state.rs` | `session_id: SessionId`, `phase: SfuSessionState` |
| `ReferenceEndpointState` | `reference-implementation/sfu/src/state.rs` | `endpoint_id: EndpointId`, `session_id: SessionId`, `phase: SfuEndpointState` |
| `ReferenceRouteState` | `reference-implementation/sfu/src/state.rs` | `route_id: RouteId`, `session_id: SessionId`, `endpoint_id: EndpointId`, `stream_id: StreamId`, `phase: SfuRouteState` |
| `ReferenceSfuOutcome` | `reference-implementation/output/src/sfu.rs` | `kind: SfuDecisionKind`, `implementation_reason: ImplementationEvidenceReason` |

`ReferenceSfuAction` は implementations-local です。frozen Kernel `SfuReferenceSet` が `session_id()` のみ public 公開するため、reference state mutation に必要な ID を運びます。variants は次に固定します。

- `AdmitParticipant { session_id: SessionId, endpoint_id: EndpointId }`
- `RejectParticipant { session_id: SessionId, endpoint_id: EndpointId }`
- `PublishStream { session_id: SessionId, endpoint_id: EndpointId, stream_id: StreamId }`
- `SubscribeRoute { session_id: SessionId, endpoint_id: EndpointId, stream_id: StreamId, route_id: RouteId }`
- `SelectRoute { session_id: SessionId, endpoint_id: EndpointId, stream_id: StreamId, route_id: RouteId }`
- `SuppressForwarding { route_id: RouteId, source: ReferenceSfuSuppressionSource }`
- `DropForwarding { route_id: RouteId }`
- `CloseSession { session_id: SessionId }`

`SuppressForwarding` の `source: Backpressure` は `SfuRouteState::SuppressedByBackpressure` に、`source: Quality` は `SfuRouteState::SuppressedByQuality` に mapping します。

**SFU action から Kernel decision への mapping**

| ReferenceSfuAction | Kernel `SfuDecisionKind` | Kernel `SfuModelKind` |
|---|---|---|
| `AdmitParticipant` | `ParticipantAdmission` | `ParticipantEndpoint` |
| `RejectParticipant` | `ParticipantAdmission` | `RejectionReason` |
| `PublishStream` | `Publication` | `Publication` |
| `SubscribeRoute` | `Subscription` | `Subscription` |
| `SelectRoute` | `RouteSelection` | `RouteCandidate` |
| `SuppressForwarding` | `Forwarding` | `ForwardingIntent` |
| `DropForwarding` | `BackpressureAction` | `BackpressureState` |
| `CloseSession` | `DegradationRecovery` | `SfuSession` |

**必須関数**

| Function | 必須シグネチャ形状 | 規則 |
|---|---|---|
| `build_kernel_sfu_item` | `(ReferenceSfuContractInput<Payload>) -> SfuContractItem<Payload>` | Kernel semantics を所有せず Kernel contract item を作る |
| `apply_reference_sfu` | `(&mut ReferenceSfuState, &ReferenceSfuAction) -> Result<ReferenceSfuOutcome, ReferenceSfuError>` | implementations-local action field を使い in-memory routing projection を更新 |
| `build_borrowed_packet_view` | `(&PacketId, &StreamId, &EndpointId, PacketHeaderSemanticView, &[u8], &[u8]) -> SfuPacketView<'_>` | packet bytes を borrowed に保つ |
| `validate_reference_sfu_state` | `(&ReferenceSfuState) -> Result<(), ReferenceSfuError>` | orphan endpoint / route を拒否 |

**packet 境界**

`build_borrowed_packet_view` は raw packet または payload bytes を copy してはなりません（MUST NOT）。返される `SfuPacketView<'packet>` は driver-owned byte slice より長生きできません。rewrite request は `PacketCopyPolicy` または `PacketRewriteTransformIntent` を返し、Kernel packet view state を mutate してはなりません。

reference state mutation は Kernel `SfuReferenceSet` から endpoint / stream / route / packet field を読んではなりません。これらは frozen Kernel contract で public accessor ではありません。reference mutation は `ReferenceSfuAction` field を implementations-local input として使い、outcome kind のみを Kernel `SfuDecisionKind` に mapping します。

### 3.6 Composition API

**必須型**

| Type | File | 必須フィールド |
|---|---|---|
| `ReferenceCompositionState` | `reference-implementation/composition/src/composition_state.rs` | `signaling: ReferenceSignalingState`, `turn: ReferenceTurnState`, `sfu: ReferenceSfuState`, `bindings: BTreeMap<String, ReferenceCrossPlaneBinding>` |
| `ReferenceCrossPlaneBinding` | `reference-implementation/composition/src/composition_state.rs` | `correlation_id: CorrelationId`, `room_id: Option<RoomId>`, `session_id: Option<SessionId>`, `allocation_id: Option<AllocationId>`, `route_id: Option<RouteId>` |
| `ReferenceCompositionOutcome` | `reference-implementation/output/src/composition.rs` | `correlation_id: CorrelationId`, `implementation_reason: ImplementationEvidenceReason` |

**必須関数**

| Function | 必須シグネチャ形状 | 規則 |
|---|---|---|
| `bind_signaling_to_turn` | `(&mut ReferenceCompositionState, CorrelationId, RoomId, AllocationId) -> Result<ReferenceCompositionOutcome, ReferenceCompositionError>` | cross-plane reference のみ記録 |
| `bind_signaling_to_sfu` | `(&mut ReferenceCompositionState, CorrelationId, RoomId, SessionId, RouteId) -> Result<ReferenceCompositionOutcome, ReferenceCompositionError>` | cross-plane reference のみ記録 |
| `validate_reference_composition_state` | `(&ReferenceCompositionState) -> Result<(), ReferenceCompositionError>` | orphan cross-plane binding を拒否 |

composition runtime bridge の exact input / step / outcome は第07章を参照。

### 3.7 error 規則

各 `Reference*Error` enum は `ImplementationEvidenceReason` に mapping すること（MUST）。いかなる reference error enum も `Unknown` を含んではなりません（MUST NOT）。

### 3.8 崩壊条件

- Kernel command private payload を implementations behavior の source of truth として扱う。
- reference state が database / queue / external storage を読む。
- `SfuPacketView` の raw packet / payload bytes を reference state に保存する。
- reference error enum に `Unknown` を追加する。
- Kernel reason catalog を implementations-local reason で拡張する。
- product plane package が `arcrtc-reference-output` 以外の reference type / function / module を input source として扱う。

---

## 4. output boundary（3-type allow-list）

本節は、product implementation が消費できる reference implementation surface を `arcrtc-reference-output` package の outcome 型に限定します。product implementation は reference source、reference internal state、fixture、local auth、runtime、composition state、reference success claim を継承しません。

### 4.1 package 境界

| Item | 固定値 |
|---|---|
| package path | `reference-implementation/output` |
| package name | `arcrtc-reference-output` |
| owner | implementations reference-output boundary |
| consumers | reference plane packages, product plane packages |
| non-consumers | Kernel |

`arcrtc-reference-output` は reference output 型だけを所有します。state mutation、fixture validation、local auth、runtime execution、composition binding、Kernel contract builder は所有しません（MUST NOT）。

### 4.2 public output 型

| Type | File | reference output fields |
|---|---|---|
| `ReferenceSignalingOutcome` | `reference-implementation/output/src/signaling.rs` | `correlation_id: CorrelationId`, `kind: SignalingEventKind`, `room_id: RoomId`, `participant_id: Option<ParticipantId>`, `implementation_reason: ImplementationEvidenceReason` |
| `ReferenceTurnOutcome` | `reference-implementation/output/src/turn.rs` | `kind: TurnDecisionKind`, `implementation_reason: ImplementationEvidenceReason` |
| `ReferenceSfuOutcome` | `reference-implementation/output/src/sfu.rs` | `kind: SfuDecisionKind`, `implementation_reason: ImplementationEvidenceReason` |
| `ReferenceCompositionOutcome` | `reference-implementation/output/src/composition.rs` | `correlation_id: CorrelationId`, `implementation_reason: ImplementationEvidenceReason` |

product plane package が policy input として受け取れる reference 型は、`ReferenceSignalingOutcome`、`ReferenceTurnOutcome`、`ReferenceSfuOutcome` の 3 つに限定します。`ReferenceCompositionOutcome` は reference composition / reference ops 用の output であり、product plane policy input には使いません（MUST NOT）。

### 4.3 product import 規則

product plane package は次の import だけを許可します。

| Product package | 許可される reference-output import |
|---|---|
| `arcrtc-product-signaling` | `arcrtc_reference_output::ReferenceSignalingOutcome` |
| `arcrtc-product-turn` | `arcrtc_reference_output::ReferenceTurnOutcome` |
| `arcrtc-product-sfu` | `arcrtc_reference_output::ReferenceSfuOutcome` |

product plane package の `arcrtc_reference_output` import は上表の exact allow-list だけに限定します。wildcard import、module import、re-export、type alias 経由 import、`ReferenceCompositionOutcome` import は禁止します（MUST NOT）。

product plane package は次を import してはなりません（MUST NOT）。

- `arcrtc_reference_output::*`
- `arcrtc_reference_output`（module import）
- `pub use arcrtc_reference_output::*`
- `pub use arcrtc_reference_output::ReferenceCompositionOutcome`
- `ReferenceCompositionOutcome`
- `arcrtc_reference_signaling`, `arcrtc_reference_turn`, `arcrtc_reference_sfu`, `arcrtc_reference_composition`, `arcrtc_reference_ops`
- `ReferenceSignalingState`, `ReferenceRoomState`, `ReferenceParticipantState`
- `ReferenceTurnState`, `ReferenceAllocationState`, `ReferencePermissionState`, `ReferenceChannelBindState`
- `ReferenceSfuState`, `ReferenceSfuSessionState`, `ReferenceEndpointState`, `ReferenceRouteState`
- `ReferenceSfuAction`, `ReferenceSfuSuppressionSource`
- `apply_reference_*`, `validate_reference_*`
- fixture / local auth modules
- reference runtime / composition modules

### 4.4 reference producer 規則

reference plane package は `arcrtc-reference-output` の outcome 型を返します。reference plane package は reference output outcome 型を `state.rs` 内で独自定義しません（MUST NOT）。

| Reference package | Producer function | Output type owner |
|---|---|---|
| `arcrtc-reference-signaling` | `apply_reference_signaling` | `arcrtc-reference-output::ReferenceSignalingOutcome` |
| `arcrtc-reference-turn` | `apply_reference_turn` | `arcrtc-reference-output::ReferenceTurnOutcome` |
| `arcrtc-reference-sfu` | `apply_reference_sfu` | `arcrtc-reference-output::ReferenceSfuOutcome` |

### 4.5 test assertion 規則

boundary tests は、product package が禁止された reference package / module / type / function / fixture / local auth / runtime / composition symbol を import したとき fail closed すること（MUST）。boundary tests は、product package が `arcrtc_reference_output` を wildcard / module path / re-export で import したとき、または `ReferenceCompositionOutcome` を import したときも fail closed すること。source import scope violation の expected failure reason は `COMMAND_SCOPE_MISMATCH` です。

### 4.6 崩壊条件

- product plane package が `arcrtc-reference-output` 以外の reference package に依存する。
- product plane package が `arcrtc_reference_output::*`、`arcrtc_reference_output` module import、re-export、type alias 経由 import、または `ReferenceCompositionOutcome` import を持つ。
- product plane package が reference state、action、fixture、local auth、runtime、composition symbol を import する。
- reference plane package が reference output outcome 型を `arcrtc-reference-output` 外で重複定義する。
- `ReferenceCompositionOutcome` を product plane policy input として扱う。
- reference success claim を product behavior / production readiness / live readiness に転用する。

---

## 5. runtime lifecycle（全状態）

本節は reference runtime、product runtime、startup、shutdown、drain、correlation propagation の lifecycle API を固定します。runtime lifecycle は implementations の process orchestration であり、Kernel semantics を所有しません。runtime lifecycle outcome は command evidence ではありません。build / test command evidence は第6節の runner helper だけが生成します。

### 5.1 runtime states

| Type | File | Variants |
|---|---|---|
| `ImplementationRuntimeState` | `reference-implementation/ops/src/runtime.rs` および `product-implementation/deployment/src/runtime.rs` | `Created`, `Starting`, `Running`, `Draining`, `Stopped`, `Failed` |
| `ImplementationShutdownMode` | `reference-implementation/ops/src/runtime.rs` および `product-implementation/rollback/src/drain.rs` | `Immediate`, `GracefulLocal`, `DrainThenStop` |
| `ImplementationRuntimePlane` | `reference-implementation/ops/src/runtime.rs` | `Signaling`, `Turn`, `Sfu`, `Composition` |
| `ReferenceRuntimeOutcome` | `reference-implementation/ops/src/runtime.rs` | `correlation_id`, `state`, `implementation_reason`, `non_claim_scope` |
| `ProductRuntimeOutcome` | `product-implementation/deployment/src/runtime.rs` | `correlation_id`, `state`, `implementation_reason`, `non_claim_scope` |

### 5.2 reference runtime API

| Function | 必須シグネチャ形状 | 規則 |
|---|---|---|
| `ReferenceRuntime::new` | `(ReferenceCompositionState) -> Self` | `Created` で開始 |
| `ReferenceRuntime::start` | `(&mut self, CorrelationId) -> Result<ReferenceRuntimeOutcome, ReferenceRuntimeError>` | transition `Created -> Starting -> Running` |
| `ReferenceRuntime::shutdown` | `(&mut self, CorrelationId, ImplementationShutdownMode) -> Result<ReferenceRuntimeOutcome, ReferenceRuntimeError>` | transition `Running -> Draining -> Stopped` |
| `ReferenceRuntime::state` | `(&self) -> ImplementationRuntimeState` | local runtime state のみ観測 |
| `ReferenceRuntime::composition` | `(&self) -> &ReferenceCompositionState` | borrowed composition state を返す |

reference runtime は public endpoint を開いてはなりません（MUST NOT）。reference runtime は production readiness または live readiness を主張してはなりません（MUST NOT）。

### 5.3 product runtime API

| Function | 必須シグネチャ形状 | 規則 |
|---|---|---|
| `ProductRuntime::new` | `(ProductRuntimeProfile) -> Self` | `Created` で開始 |
| `ProductRuntime::select` | `(&ProductRuntimeProfile) -> ProductRuntimeSelection` | readiness claim なし |
| `ProductRuntime::start` | `(&mut self, CorrelationId) -> Result<ProductRuntimeOutcome, ProductRuntimeError>` | bounded product runtime start outcome |
| `ProductRuntime::drain` | `(&mut self, ProductDrainPlan) -> Result<ProductRuntimeOutcome, ProductRuntimeError>` | bounded drain outcome |
| `ProductRuntime::restore` | `(&mut self, ProductRestorePlan) -> Result<ProductRuntimeOutcome, ProductRuntimeError>` | bounded restore outcome |

`ProductionDeferred` または `LiveDeferred` environment class を持つ product runtime は readiness claim に対し fail closed すること（MUST）。

### 5.4 correlation propagation

すべての runtime command は 1 つの `CorrelationId` を運ぶこと（MUST）。同一 correlation id は次に出現すること: runtime lifecycle outcome、plane-specific evidence record、composition binding record。runtime action が correlation id 相当を欠く場合、adopted outcome を作る前に fail すること（fail-closed）。

### 5.5 timeout / cancellation 規則

runtime timeout と cancellation は implementations-local execution facts です。

| Event | 必須 reason |
|---|---|
| task failed before start | `RUNTIME_EXECUTOR_ERROR` |
| shutdown lacks evidence fields | `EVIDENCE_FIELDS_INCOMPLETE` |
| command targets wrong working directory | `COMMAND_SCOPE_MISMATCH` |
| readiness command lacks its readiness admission record | `READINESS_NOT_ADMITTED` |

### 5.6 崩壊条件

- runtime start success を production readiness として扱う。
- runtime drain success を live readiness として扱う。
- runtime lifecycle state を Kernel state authority として扱う。
- correlation id のない runtime outcome を採用する。
- runtime lifecycle outcome を build / test command evidence として採用する。
- runtime API が public endpoint availability を主張する。

---

## 6. command evidence runner（実行と evidence 生成手順）

本節は implementations command evidence runner、output path、JSON writer、validation command を固定します。command evidence runner は evidence record を生成するだけであり、build success、test pass、benchmark pass、readiness success を自動主張しません。

### 6.1 runner ownership

| Runner | Owner package | File | Command class |
|---|---|---|---|
| reference build command evidence helper | `arcrtc-reference-ops` | `src/runtime.rs` | bounded build command 呼び出し時 `Build` |
| product monitoring evidence helper | `arcrtc-product-monitoring` | `src/evidence.rs` | `Build` / `Test` |
| benchmark evidence helper | benchmark package | `src/evidence.rs` | `Benchmark` |
| real-device evidence helper | real-device package | `src/evidence.rs` | `RealDevice` |
| production readiness evidence matrix | production-readiness package | `tests/production-readiness/tests/*.rs` | `ProductionReadiness` |
| live readiness evidence matrix | live package | `tests/live/tests/*.rs` | `LiveReadiness` |

shared evidence record / reason type owner は `arcrtc-implementation-evidence` です。implementations command root / target root / evidence root constants は `implementation-support/evidence/src/validation.rs` が所有し、`arcrtc-implementation-evidence` が re-export します。いかなる runner も `implementations/target/` 外に evidence を書いてはなりません（MUST NOT）。

### 6.2 output path 規則

| Evidence class | Output directory |
|---|---|
| format | `target/implementation-evidence/format/` |
| build | `target/implementation-evidence/build/` |
| test | `target/implementation-evidence/test/` |
| benchmark | `target/implementation-evidence/benchmark/` |
| real-device | `target/implementation-evidence/real-device/` |
| production readiness | `target/implementation-evidence/production-readiness/` |
| live readiness | `target/implementation-evidence/live-readiness/` |

output filename は `{correlation_id}.json` に固定します。writer は single filename component として安全でない `correlation_id` を拒否すること。writer は filename を silent に rewrite / sanitize してはなりません（`{correlation_id}.json` evidence binding を壊すため）。

### 6.3 JSON writer 規則

JSON writer は `ImplementationEvidenceRecord` を snake_case field 名と所定の wire 値で serialize すること。書き込み前の必須 validation は次です。

- `arcrtc-implementation-evidence` の `validate_evidence_record(&record)` が `Ok(())` を返す。
- output path が許可された evidence directory 配下である。
- reference ops writer は command class と owner binding が reference runner 表に一致する `Reference` / `Ops` record のみ受理する。
- product monitoring evidence helper は command class が `Build` または `Test`、target package が `arcrtc-product-*`、target scope が `product-implementation/` 配下である `Product` record のみ受理する。
- build evidence helper は `cargo build --workspace --all-targets` 以外の command を拒否する。
- test evidence helper は `cargo test --workspace --all-targets` 以外の command を拒否する。
- reference ops writer は `Test` evidence record を受理してはならない。reference ops は build evidence のみを所有する。

base validation failure は `EVIDENCE_FIELDS_INCOMPLETE`、`COMMAND_SCOPE_MISMATCH`、`READINESS_NOT_ADMITTED` のいずれかを返すこと。benchmark extension validation failure は追加で `BENCHMARK_SCOPE_MISMATCH` を、real-device extension validation failure は追加で `REAL_DEVICE_SCOPE_MISMATCH` を返し得ます。

### 6.4 初期 validation command

| Command | Working directory | Expected result |
|---|---|---|
| placeholder / forbidden 語句検出の `rg`（`{PROJECT}` / `{MISSION}` / `TODO` / `TBD` / 未定義 等） | `implementations` | no output |
| boundary 語句検出の `rg`（`v0.2/userland` / `userland` / `adapters/` / `apps/` 等） | `implementations` | no output |
| `UNKNOWN` / `Unknown` / `Other` / `raw String` の `rg` | `implementations` | prohibition text のみ |

test / benchmark / real-device command 呼び出しは explicit manifest path を使用します。

### 6.5 claim boundary

| Runner result | May support | MUST NOT support |
|---|---|---|
| evidence JSON write success | evidence shape generation | command target success |
| build command evidence | build command result | behavior correctness |
| test command evidence | tested behavior | production readiness |
| benchmark command evidence | measurement record | threshold satisfaction |
| real-device command evidence | bounded device command result | live readiness |

### 6.6 崩壊条件

- evidence writer が空の correlation id を受理する。
- evidence writer が `target/implementation-evidence/` 外に書く。
- runner が必須 field 欠落の JSON を emit する。
- runner が `validate_evidence_record` を呼ばず JSON を書く。
- runner が command execution success を readiness claim に変換する。
- runner が Kernel working directory command result を implementations evidence として採用する。

---

## 7. 本章の不変条件（要約）

1. reference state / fixture は local deterministic projection であり、Kernel state authority、database、production state、readiness evidence ではありません。
2. absent state と rejected state は同じ enum variant / mutation result で表現してはなりません（MUST NOT）。
3. すべての state map key は Kernel `as_str()` 値由来であり、random / provider / wall-clock / memory / raw payload / `Debug` 表現を使いません。
4. product が消費できる reference 型は 3-type allow-list（`ReferenceSignalingOutcome` / `ReferenceTurnOutcome` / `ReferenceSfuOutcome`）に限定します。`ReferenceCompositionOutcome` は product policy input ではありません。
5. すべての outcome / runtime command は correlation id を運び、reference error enum は `Unknown` を持ちません。
6. command evidence runner は evidence record 生成のみを行い、build / test / benchmark / readiness success を自動主張しません（fail-closed）。
