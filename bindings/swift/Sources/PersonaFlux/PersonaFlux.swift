import Foundation
import CPersonaFlux

public enum PersonaFluxError: Error, Equatable {
    case invalidArgument(String)
    case notFound(String)
    case invalidState(String)
    case bufferTooSmall(String)
    case serialization(String)
    case versionMismatch(String)
    case internalError(String)
    case unknown(code: Int32, message: String)
}

public struct Pad: Equatable, Sendable {
    public let pleasure: Float
    public let arousal: Float
    public let dominance: Float
    fileprivate init(_ value: pf_pad_t) { pleasure = value.pleasure; arousal = value.arousal; dominance = value.dominance }
}

public struct RelationshipLookup: Equatable, Sendable {
    public let present: Bool
    public let source: UInt32
    public let affinity: Float
    fileprivate init(_ value: pf_relationship_lookup_t) { present = value.present != 0; source = value.source; affinity = value.affinity }
}

public struct EvaluationResult: Equatable, Sendable {
    public let concern: Float
    public let effectiveConfidence: Float
    public let rawAffinityDelta: Float
    public let rawPadDelta: Pad
    public let eventIntensity: Float
    public let memorySalience: Float
    public let memoryKind: UInt32
    fileprivate init(_ value: pf_evaluation_result_t) {
        concern = value.concern; effectiveConfidence = value.effective_confidence
        rawAffinityDelta = value.raw_affinity_delta; rawPadDelta = Pad(value.raw_pad_delta)
        eventIntensity = value.event_intensity; memorySalience = value.memory_salience
        memoryKind = value.memory_kind
    }
}

public struct MemoryRecord: Equatable, Sendable {
    public let observer: UInt64
    public let deedID: UInt64
    public let actor: UInt64
    public let target: UInt64?
    public let impact: Float
    public let aggression: Float
    public let salience: Float
    public let kind: UInt32
    public let createdTick: UInt64
    public let expiresAt: UInt64
    fileprivate init(_ value: pf_memory_record_t) {
        observer = value.observer; deedID = value.deed_id; actor = value.actor
        target = value.has_target != 0 ? value.target : nil
        impact = value.impact; aggression = value.aggression; salience = value.salience
        kind = value.kind; createdTick = value.created_tick; expiresAt = value.expires_at
    }
}

public struct DirectWitnessDeed: Sendable {
    public let deedID: UInt64
    public let observer: UInt64
    public let actor: UInt64
    public let target: UInt64?
    public let impact: Float
    public let aggression: Float
    public let threatensObserver: Bool
    public init(deedID: UInt64, observer: UInt64, actor: UInt64, target: UInt64?, impact: Float, aggression: Float, threatensObserver: Bool) {
        self.deedID = deedID; self.observer = observer; self.actor = actor; self.target = target
        self.impact = impact; self.aggression = aggression; self.threatensObserver = threatensObserver
    }
}

public struct DirectWitnessOutcome: Equatable, Sendable {
    public let deedID: UInt64
    public let observer: UInt64
    public let actor: UInt64
    public let target: UInt64?
    public let relationship: RelationshipLookup
    public let evaluation: EvaluationResult
    public let previousAffinity: Float
    public let currentAffinity: Float
    public let previousPad: Pad
    public let currentPad: Pad
    public let memory: MemoryRecord?
    fileprivate init(_ value: pf_direct_witness_outcome_t) {
        deedID = value.deed_id; observer = value.observer; actor = value.actor
        target = value.has_target != 0 ? value.target : nil
        relationship = RelationshipLookup(value.relationship); evaluation = EvaluationResult(value.evaluation)
        previousAffinity = value.previous_affinity; currentAffinity = value.current_affinity
        previousPad = Pad(value.previous_pad); currentPad = Pad(value.current_pad)
        memory = value.has_memory != 0 ? MemoryRecord(value.memory) : nil
    }
}

public enum SubmissionResult: Equatable, Sendable {
    case applied(DirectWitnessOutcome)
    case duplicate(observer: UInt64, deedID: UInt64)
}

public struct SimulationEvent: Equatable, Sendable {
    public let kind: UInt32
    public let factionID: UInt64
    public let memberID: UInt64
    public let deedID: UInt64
    public let observer: UInt64
    public let actor: UInt64
    public let target: UInt64?
    public let relationshipLayer: UInt32
    public let relationshipSourceID: UInt64
    public let relationshipTargetID: UInt64
    public let previousPresent: Bool
    public let currentPresent: Bool
    public let previousAffinity: Float
    public let currentAffinity: Float
    public let evaluation: EvaluationResult
    public let previousPad: Pad
    public let currentPad: Pad
    public let memoryKind: UInt32
    public let previousMemoryKind: UInt32
    public let currentMemoryKind: UInt32
    public let salience: Float
    public let previousTick: UInt64
    public let currentTick: UInt64
    fileprivate init(_ value: pf_event_t) {
        kind = value.kind; factionID = value.faction_id; memberID = value.member_id; deedID = value.deed_id
        observer = value.observer; actor = value.actor; target = value.has_target != 0 ? value.target : nil
        relationshipLayer = value.relationship_layer; relationshipSourceID = value.relationship_source_id
        relationshipTargetID = value.relationship_target_id; previousPresent = value.previous_present != 0
        currentPresent = value.current_present != 0; previousAffinity = value.previous_affinity
        currentAffinity = value.current_affinity; evaluation = EvaluationResult(value.evaluation)
        previousPad = Pad(value.previous_pad); currentPad = Pad(value.current_pad)
        memoryKind = value.memory_kind; previousMemoryKind = value.previous_memory_kind
        currentMemoryKind = value.current_memory_kind; salience = value.salience
        previousTick = value.previous_tick; currentTick = value.current_tick
    }
}

public final class Simulation {
    public static let abiVersion: UInt32 = PF_ABI_VERSION
    public static let modelVersion: UInt32 = PF_MODEL_VERSION
    public static func queryABIVersion() throws -> UInt32 { var value: UInt32 = 0; try check(pf_api_version(&value)); return value }
    public static func queryModelVersion() throws -> UInt32 { var value: UInt32 = 0; try check(pf_model_version(&value)); return value }
    private var handle: OpaquePointer?

    public init(randomSeed: UInt64) throws {
        var raw: OpaquePointer?
        try check(pf_simulation_create(randomSeed, &raw))
        guard raw != nil else { throw PersonaFluxError.internalError("simulation creation returned a null handle") }
        handle = raw
    }

    deinit { close() }
    public func close() { if let value = handle { pf_simulation_destroy(value); handle = nil } }

    public var randomSeed: UInt64 { get throws { try withHandle { value in var output: UInt64 = 0; try check(pf_simulation_random_seed(value, &output)); return output } } }
    public var currentTick: UInt64 { get throws { try withHandle { value in var output: UInt64 = 0; try check(pf_simulation_current_tick(value, &output)); return output } } }

    public func addFaction(name: String) throws -> UInt64 {
        try withHandle { value in
            let bytes = Array(name.utf8)
            return try bytes.withUnsafeBytes { raw in
                var output: UInt64 = 0
                try check(pf_faction_add(value, raw.bindMemory(to: UInt8.self).baseAddress, UInt32(bytes.count), &output))
                return output
            }
        }
    }

    public func factionName(_ faction: UInt64) throws -> String {
        try withHandle { value in
            var length: UInt32 = 0
            let sizeCode = pf_faction_name_get(value, faction, nil, 0, &length)
            if sizeCode != PF_OK && sizeCode != PF_BUFFER_TOO_SMALL { try check(sizeCode) }
            guard length > 0 else { return "" }
            var bytes = [UInt8](repeating: 0, count: Int(length))
            try check(bytes.withUnsafeMutableBytes { raw in
                pf_faction_name_get(value, faction, raw.bindMemory(to: UInt8.self).baseAddress, length, &length)
            })
            guard let result = String(bytes: bytes, encoding: .utf8) else { throw PersonaFluxError.internalError("core returned invalid UTF-8") }
            return result
        }
    }

    public func addMember(to faction: UInt64) throws -> UInt64 { try withHandle { value in var output: UInt64 = 0; try check(pf_member_add(value, faction, &output)); return output } }
    public func memberState(_ member: UInt64) throws -> (factionID: UInt64, pad: Pad) { try withHandle { value in var output = pf_member_state_t(struct_size: UInt32(MemoryLayout<pf_member_state_t>.size), api_version: PF_ABI_VERSION, faction_id: 0, pad: pf_pad_t(pleasure: 0, arousal: 0, dominance: 0)); try check(pf_member_state_get(value, member, &output, UInt32(MemoryLayout.size(ofValue: output)))); return (output.faction_id, Pad(output.pad)) } }
    public func memberAffinity(observer: UInt64, actor: UInt64) throws -> Float { try withHandle { value in var output: Float = 0; try check(pf_member_affinity_get(value, observer, actor, &output)); return output } }

    public func setMemberRelationship(observer: UInt64, target: UInt64, affinity: Float) throws { try withHandle { value in try check(pf_relationship_member_to_member_set(value, observer, target, affinity)) } }
    public func clearMemberRelationship(observer: UInt64, target: UInt64) throws { try withHandle { value in try check(pf_relationship_member_to_member_clear(value, observer, target)) } }
    public func setFactionMemberRelationship(faction: UInt64, member: UInt64, affinity: Float) throws { try withHandle { value in try check(pf_relationship_faction_to_member_set(value, faction, member, affinity)) } }
    public func clearFactionMemberRelationship(faction: UInt64, member: UInt64) throws { try withHandle { value in try check(pf_relationship_faction_to_member_clear(value, faction, member)) } }
    public func setFactionRelationship(source: UInt64, target: UInt64, affinity: Float) throws { try withHandle { value in try check(pf_relationship_faction_to_faction_set(value, source, target, affinity)) } }
    public func clearFactionRelationship(source: UInt64, target: UInt64) throws { try withHandle { value in try check(pf_relationship_faction_to_faction_clear(value, source, target)) } }

    private func relationship(_ call: (OpaquePointer, UnsafeMutablePointer<pf_relationship_lookup_t>, UInt32) -> Int32) throws -> RelationshipLookup { try withHandle { value in var output = pf_relationship_lookup_t(struct_size: UInt32(MemoryLayout<pf_relationship_lookup_t>.size), api_version: PF_ABI_VERSION, present: 0, reserved: (0, 0, 0), source: 0, affinity: 0); try check(call(value, &output, UInt32(MemoryLayout.size(ofValue: output)))); return RelationshipLookup(output) } }
    public func memberRelationship(observer: UInt64, target: UInt64) throws -> RelationshipLookup { try relationship { value, output, size in pf_relationship_member_to_member_get(value, observer, target, output, size) } }
    public func factionMemberRelationship(faction: UInt64, member: UInt64) throws -> RelationshipLookup { try relationship { value, output, size in pf_relationship_faction_to_member_get(value, faction, member, output, size) } }
    public func factionRelationship(source: UInt64, target: UInt64) throws -> RelationshipLookup { try relationship { value, output, size in pf_relationship_faction_to_faction_get(value, source, target, output, size) } }
    public func effectiveMemberRelationship(observer: UInt64, target: UInt64) throws -> RelationshipLookup { try relationship { value, output, size in pf_relationship_effective_member_get(value, observer, target, output, size) } }

    public func submit(_ deed: DirectWitnessDeed) throws -> SubmissionResult { try withHandle { value in var raw = makeDeed(deed); var output = pf_submission_result_t(); try check(pf_simulation_submit_direct_witness(value, &raw, &output, UInt32(MemoryLayout.size(ofValue: output)))); return makeSubmission(output) } }
    public func submitBatch(_ deeds: [DirectWitnessDeed]) throws -> [SubmissionResult] {
        try withHandle { value in
            var raw = deeds.map(makeDeed); let size = MemoryLayout<pf_submission_result_t>.stride
            let inputCount = UInt32(raw.count)
            let outputCapacity = UInt32(raw.count)
            var output = [pf_submission_result_t](repeating: pf_submission_result_t(), count: raw.count); var count: UInt32 = 0; var index: UInt32 = UInt32.max
            let code = raw.withUnsafeMutableBytes { inputs in
                output.withUnsafeMutableBytes { outputs in
                    pf_simulation_submit_direct_witness_batch(value, inputs.bindMemory(to: pf_direct_witness_deed_t.self).baseAddress, inputCount, outputs.bindMemory(to: pf_submission_result_t.self).baseAddress, outputCapacity, UInt32(size), &count, &index)
                }
            }
            try check(code, errorIndex: index == UInt32.max ? nil : index)
            return output.prefix(Int(count)).map(makeSubmission)
        }
    }

    public func step(_ delta: UInt64) throws { try withHandle { value in try check(pf_simulation_step(value, delta)) } }
    public func advance(to tick: UInt64) throws { try withHandle { value in try check(pf_simulation_advance_to(value, tick)) } }
    public func memory(observer: UInt64, deedID: UInt64) throws -> MemoryRecord? { try withHandle { value in var record = pf_memory_record_t(); var present: UInt8 = 0; try check(pf_memory_get(value, observer, deedID, &record, UInt32(MemoryLayout.size(ofValue: record)), &present)); return present != 0 ? MemoryRecord(record) : nil } }
    public func memories(for observer: UInt64) throws -> [MemoryRecord] { try withHandle { value in var count: UInt32 = 0; try check(pf_memories_count(value, observer, &count)); if count == 0 { return [] }; var records = [pf_memory_record_t](repeating: pf_memory_record_t(), count: Int(count)); var written: UInt32 = 0; try check(records.withUnsafeMutableBytes { raw in pf_memories_read(value, observer, raw.bindMemory(to: pf_memory_record_t.self).baseAddress, count, UInt32(MemoryLayout<pf_memory_record_t>.stride), &written) }); return records.prefix(Int(written)).map(MemoryRecord.init) } }
    public func events() throws -> [SimulationEvent] { try withHandle { value in var count: UInt32 = 0; try check(pf_events_count(value, &count)); if count == 0 { return [] }; var events = [pf_event_t](repeating: pf_event_t(), count: Int(count)); var written: UInt32 = 0; try check(events.withUnsafeMutableBytes { raw in pf_events_read(value, raw.bindMemory(to: pf_event_t.self).baseAddress, count, UInt32(MemoryLayout<pf_event_t>.stride), &written) }); return events.prefix(Int(written)).map(SimulationEvent.init) } }
    public func clearEvents() throws { try withHandle { value in try check(pf_events_clear(value)) } }

    private func withHandle<T>(_ body: (OpaquePointer) throws -> T) throws -> T { guard let value = handle else { throw PersonaFluxError.invalidState("simulation is closed") }; return try body(value) }
    private func makeDeed(_ value: DirectWitnessDeed) -> pf_direct_witness_deed_t { pf_direct_witness_deed_t(struct_size: UInt32(MemoryLayout<pf_direct_witness_deed_t>.size), api_version: PF_ABI_VERSION, deed_id: value.deedID, observer: value.observer, actor: value.actor, target: value.target ?? 0, impact: value.impact, aggression: value.aggression, has_target: value.target == nil ? 0 : 1, threatens_observer: value.threatensObserver ? 1 : 0, reserved: (0, 0)) }
    private func makeSubmission(_ value: pf_submission_result_t) -> SubmissionResult { value.kind == PF_SUBMISSION_APPLIED ? .applied(DirectWitnessOutcome(value.outcome)) : .duplicate(observer: value.observer, deedID: value.deed_id) }
}

private func check(_ code: Int32, errorIndex: UInt32? = nil) throws {
    guard code != Int32(PF_OK) else { return }
    var length: UInt32 = 0; _ = pf_last_error_message_copy(nil, 0, &length)
    var bytes = [UInt8](repeating: 0, count: Int(length)); if length > 0 { _ = bytes.withUnsafeMutableBytes { pf_last_error_message_copy($0.bindMemory(to: UInt8.self).baseAddress, length, &length) } }
    let message = String(bytes: bytes, encoding: .utf8) ?? "PersonaFlux operation failed"
    let diagnostic = errorIndex.map { "batch index \($0): \(message)" } ?? message
    switch Int(code) {
    case PF_INVALID_ARGUMENT: throw PersonaFluxError.invalidArgument(diagnostic)
    case PF_NOT_FOUND: throw PersonaFluxError.notFound(diagnostic)
    case PF_INVALID_STATE: throw PersonaFluxError.invalidState(diagnostic)
    case PF_BUFFER_TOO_SMALL: throw PersonaFluxError.bufferTooSmall(diagnostic)
    case PF_SERIALIZATION_ERROR: throw PersonaFluxError.serialization(diagnostic)
    case PF_VERSION_MISMATCH: throw PersonaFluxError.versionMismatch(diagnostic)
    case PF_INTERNAL_ERROR: throw PersonaFluxError.internalError(diagnostic)
    default: throw PersonaFluxError.unknown(code: code, message: diagnostic)
    }
}
