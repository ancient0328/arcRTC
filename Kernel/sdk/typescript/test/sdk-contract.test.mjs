import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import * as sdk from "../src/index.ts";

const packageJson = JSON.parse(readFileSync(new URL("../package.json", import.meta.url), "utf8"));
const source = readFileSync(new URL("../src/index.ts", import.meta.url), "utf8");

const expectedCommands = [
  "JoinRoom",
  "LeaveRoom",
  "SendOffer",
  "SendAnswer",
  "SendIceCandidate",
  "RequestTurnCredential",
  "AcknowledgeForward",
];

const expectedEvents = [
  "Joined",
  "Rejected",
  "ParticipantJoined",
  "ParticipantLeft",
  "OfferReceived",
  "AnswerReceived",
  "IceCandidateReceived",
  "TurnCredentialAvailable",
  "ProtocolViolation",
];

const expectedPublicCommandTypes = [
  "ArcRtcJoinCommand",
  "ArcRtcLeaveCommand",
  "ArcRtcOfferCommand",
  "ArcRtcAnswerCommand",
  "ArcRtcIceCandidateCommand",
  "ArcRtcReconnectCommand",
];

const expectedPublicEventTypes = [
  "ArcRtcJoinAcceptedEvent",
  "ArcRtcJoinRejectedEvent",
  "ArcRtcParticipantLeftEvent",
  "ArcRtcNegotiationRequiredEvent",
  "ArcRtcIceCandidateReceivedEvent",
  "ArcRtcSessionTimedOutEvent",
];

const expectedSdkFailureCodes = [
  "CredentialRejected",
  "JoinRejected",
  "MembershipConflict",
  "NegotiationRejected",
  "IceCandidateRejected",
  "Timeout",
  "TransportUnavailable",
  "ProtocolViolation",
  "VersionMismatch",
  "InternalInvariantViolation",
];

const requiredPublicFields = [
  "roomId",
  "participantId",
  "credentialRef",
  "sdpRef",
  "candidateRef",
  "sessionRef",
  "reasonCode",
];

function extractConstArray(name) {
  const start = `export const ${name} = [`;
  const startIndex = source.indexOf(start);
  assert.notEqual(startIndex, -1, `${name} array must exist`);
  const bodyStart = startIndex + start.length;
  const bodyEnd = source.indexOf("] as const", bodyStart);
  assert.notEqual(bodyEnd, -1, `${name} array must close with as const`);
  return [...source.slice(bodyStart, bodyEnd).matchAll(/"([^"]+)"/g)].map((entry) => entry[1]);
}

function extractProjectionEntries() {
  const match = source.match(
    /export const arcrtcTypeScriptSdkProjectionEntries = \[([\s\S]*?)\] as const/,
  );
  assert.ok(match, "projection entries must exist");
  return [...match[1].matchAll(/\{([\s\S]*?)\},/g)].map((entry) => {
    const body = entry[1];
    const field = (name) => {
      const fieldMatch = body.match(new RegExp(`${name}: "([^"]+)"`));
      assert.ok(fieldMatch, `${name} must exist in projection entry`);
      return fieldMatch[1];
    };
    return {
      sourceSignalingKind: field("sourceSignalingKind"),
      sourceKindClass: field("sourceKindClass"),
      reconnectRelation: field("reconnectRelation"),
      serverReasonPreservation: field("serverReasonPreservation"),
      unsupportedSurfaceBehavior: field("unsupportedSurfaceBehavior"),
    };
  });
}

function extractTypeAliasBody(name) {
  const match = source.match(new RegExp(`export type ${name} =([\\s\\S]*?);\\n`));
  assert.ok(match, `${name} type alias must exist`);
  return match[1];
}

function extractTypeUnionValues(name) {
  return [...extractTypeAliasBody(name).matchAll(/"([^"]+)"/g)].map((entry) => entry[1]);
}

test("TypeScript SDK remains a Signaling-only public surface", () => {
  assert.equal(packageJson.arcrtc.ownerLayer, "sdk");
  assert.match(packageJson.arcrtc.packageRole, /signaling-only/i);
  assert.match(source, /Signaling/);
  assert.match(source, /regulated/i);
  assert.equal(sdk.arcrtcTypeScriptSdkSurface, "signaling-only");
  assert.equal(sdk.arcrtcTypeScriptSdkPlatform, "typescript");
  assert.equal(sdk.arcrtcSignalingContractVersion, "v0.2");
});

test("TypeScript SDK does not import driver or regulated internals", () => {
  assert.doesNotMatch(source, /drivers\//);
  assert.doesNotMatch(source, /regulated\//);
  assert.doesNotMatch(source, /arcrtc-driver/);
});

test("TypeScript SDK projection covers every Signaling command and event exactly once", () => {
  assert.deepEqual(extractConstArray("signalingCommandKinds"), expectedCommands);
  assert.deepEqual(extractConstArray("signalingEventKinds"), expectedEvents);
  assert.deepEqual([...sdk.signalingCommandKinds], expectedCommands);
  assert.deepEqual([...sdk.signalingEventKinds], expectedEvents);

  const entries = extractProjectionEntries();
  assert.equal(entries.length, expectedCommands.length + expectedEvents.length);
  assert.deepEqual(
    entries.filter((entry) => entry.sourceKindClass === "command").map((entry) => entry.sourceSignalingKind),
    expectedCommands,
  );
  assert.deepEqual(
    entries.filter((entry) => entry.sourceKindClass === "event").map((entry) => entry.sourceSignalingKind),
    expectedEvents,
  );
  assert.equal(
    sdk.arcrtcTypeScriptSdkProjection.entries.length,
    expectedCommands.length + expectedEvents.length,
  );
  assert.deepEqual(
    sdk.arcrtcTypeScriptSdkProjection.entries
      .filter((entry) => entry.sourceKindClass === "command")
      .map((entry) => entry.sourceSignalingKind),
    expectedCommands,
  );
  assert.deepEqual(
    sdk.arcrtcTypeScriptSdkProjection.entries
      .filter((entry) => entry.sourceKindClass === "event")
      .map((entry) => entry.sourceSignalingKind),
    expectedEvents,
  );
});

test("TypeScript SDK exposes the fixed Signaling-only public command and event names", () => {
  for (const commandName of expectedPublicCommandTypes) {
    assert.match(source, new RegExp(`export type ${commandName} =`), commandName);
  }
  for (const eventName of expectedPublicEventTypes) {
    assert.match(source, new RegExp(`export type ${eventName} =`), eventName);
  }

  for (const fieldName of requiredPublicFields) {
    assert.match(source, new RegExp(`readonly ${fieldName}\\??: string`), fieldName);
  }
});

test("TypeScript SDK exposes the fixed failure-code and version constants", () => {
  assert.deepEqual(extractTypeUnionValues("ArcRtcSdkFailureCode"), expectedSdkFailureCodes);
  assert.equal(sdk.ARCRTC_SIGNALING_PROTOCOL_VERSION, "v0.2");
  assert.equal(sdk.ARCRTC_SDK_SEMVER, "0.2.0");
  assert.equal(sdk.ARCRTC_SDK_PACKAGE_NAME, packageJson.name);
  assert.equal(sdk.ARCRTC_SDK_PACKAGE_NAME, "@arcrtc/sdk-typescript");
});

test("SDK reconnect and generated-artifact rules remain fail-closed", () => {
  const entries = extractProjectionEntries();
  const join = entries.find((entry) => entry.sourceSignalingKind === "JoinRoom");
  assert.equal(join.reconnectRelation, "signaling_rejoin_command");
  const runtimeJoin = sdk.arcrtcTypeScriptSdkProjection.entries.find(
    (entry) => entry.sourceSignalingKind === "JoinRoom",
  );
  assert.equal(runtimeJoin.reconnectRelation, "signaling_rejoin_command");

  for (const event of entries.filter((entry) => entry.sourceKindClass === "event")) {
    assert.equal(event.reconnectRelation, "event_stream_resubscribe", event.sourceSignalingKind);
  }

  assert.match(source, /generatedArtifactIsSemanticAuthority:\s*false/);
  assert.doesNotMatch(source, /generatedArtifactIsSemanticAuthority:\s*true/);
  assert.equal((source.match(/sourceContract:\s*"SIGNALING_CONTRACT_CANONICAL"/g) ?? []).length, 2);
  assert.equal(sdk.arcrtcTypeScriptSdkProjection.generatedArtifactIsSemanticAuthority, false);
  assert.equal(sdk.arcrtcTypeScriptSdkProjection.sourceContract, "SIGNALING_CONTRACT_CANONICAL");
});

test("SDK out-of-scope catalog excludes media auth issuance and regulated workflow", () => {
  const outOfScope = extractConstArray("sdkOutOfScopeFeatures");
  for (const feature of [
    "PeerConnection",
    "media_capture",
    "media_rendering",
    "screen_share",
    "recording",
    "auth_issuance",
    "regulated_workflow",
    "medical_data_model",
  ]) {
    assert.ok(outOfScope.includes(feature), feature);
    assert.ok(sdk.sdkOutOfScopeFeatures.includes(feature), feature);
  }
});
