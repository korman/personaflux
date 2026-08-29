package com.personaflux

import java.nio.ByteBuffer
import java.nio.ByteOrder

public enum class ResultCode(public val value: Int) { OK(0), INVALID_ARGUMENT(1), NOT_FOUND(2), INVALID_STATE(3), BUFFER_TOO_SMALL(4), SERIALIZATION_ERROR(5), VERSION_MISMATCH(6), INTERNAL_ERROR(255) }
public class PersonaFluxException(public val code: ResultCode, message: String, public val errorIndex: UInt? = null) : Exception(message)
public data class Pad(val pleasure: Float, val arousal: Float, val dominance: Float)
public data class RelationshipLookup(val present: Boolean, val source: UInt, val affinity: Float)
public data class EvaluationResult(val concern: Float, val effectiveConfidence: Float, val rawAffinityDelta: Float, val rawPadDelta: Pad, val eventIntensity: Float, val memorySalience: Float, val memoryKind: UInt)
public data class MemoryRecord(val observer: ULong, val deedId: ULong, val actor: ULong, val target: ULong?, val impact: Float, val aggression: Float, val salience: Float, val kind: UInt, val createdTick: ULong, val expiresAt: ULong)
public data class MemberState(val factionId: ULong, val pad: Pad)
public data class DirectWitnessDeed(val deedId: ULong, val observer: ULong, val actor: ULong, val target: ULong?, val impact: Float, val aggression: Float, val threatensObserver: Boolean)
public data class DirectWitnessOutcome(val deedId: ULong, val observer: ULong, val actor: ULong, val target: ULong?, val relationship: RelationshipLookup, val evaluation: EvaluationResult, val previousAffinity: Float, val currentAffinity: Float, val previousPad: Pad, val currentPad: Pad, val memory: MemoryRecord?)
public sealed class SubmissionResult { public data class Applied(val outcome: DirectWitnessOutcome) : SubmissionResult(); public data class Duplicate(val observer: ULong, val deedId: ULong) : SubmissionResult() }
public data class SimulationEvent(val kind: UInt, val factionId: ULong, val memberId: ULong, val deedId: ULong, val observer: ULong, val actor: ULong, val target: ULong?, val relationshipLayer: UInt, val relationshipSourceId: ULong, val relationshipTargetId: ULong, val previousPresent: Boolean, val currentPresent: Boolean, val previousAffinity: Float, val currentAffinity: Float, val evaluation: EvaluationResult, val previousPad: Pad, val currentPad: Pad, val memoryKind: UInt, val previousMemoryKind: UInt, val currentMemoryKind: UInt, val salience: Float, val previousTick: ULong, val currentTick: ULong)

public class Simulation(randomSeed: ULong) : AutoCloseable {
    public companion object {
        public const val ABI_VERSION: UInt = 0u
        public const val MODEL_VERSION: UInt = 1u
        public fun queryAbiVersion(): UInt { val output = IntArray(1); check(Native.nativeApiVersion(output)); return output[0].toUInt() }
        public fun queryModelVersion(): UInt { val output = IntArray(1); check(Native.nativeModelVersion(output)); return output[0].toUInt() }
    }
    private var handle: Long
    init { val output = LongArray(1); check(Native.nativeCreate(randomSeed.toLong(), output)); handle = output[0]; require(handle != 0L) { "native simulation handle is null" } }

    public val randomSeed: ULong get() { val output = LongArray(1); check(Native.nativeRandomSeed(requireHandle(), output)); return output[0].toULong() }
    public val currentTick: ULong get() { val output = LongArray(1); check(Native.nativeCurrentTick(requireHandle(), output)); return output[0].toULong() }
    public fun close() { if (handle != 0L) { Native.nativeDestroy(handle); handle = 0L } }

    public fun addFaction(name: String): ULong { val output = LongArray(1); check(Native.nativeAddFaction(requireHandle(), name.toByteArray(Charsets.UTF_8), output)); return output[0].toULong() }
    public fun factionName(faction: ULong): String { val code = IntArray(1); val bytes = Native.nativeFactionName(requireHandle(), faction.toLong(), code); check(code[0]); return bytes?.toString(Charsets.UTF_8) ?: "" }
    public fun addMember(faction: ULong): ULong { val output = LongArray(1); check(Native.nativeAddMember(requireHandle(), faction.toLong(), output)); return output[0].toULong() }
    public fun memberState(member: ULong): MemberState { val output = buffer(Layout.MEMBER_STATE); check(Native.nativeMemberState(requireHandle(), member.toLong(), output)); return MemberState(output.getLong(8).toULong(), Pad(output.getFloat(16), output.getFloat(20), output.getFloat(24))) }
    public fun memberAffinity(observer: ULong, actor: ULong): Float { val output = FloatArray(1); check(Native.nativeMemberAffinity(requireHandle(), observer.toLong(), actor.toLong(), output)); return output[0] }

    public fun setMemberRelationship(observer: ULong, target: ULong, affinity: Float) = check(Native.nativeMemberRelationshipSet(requireHandle(), observer.toLong(), target.toLong(), affinity))
    public fun clearMemberRelationship(observer: ULong, target: ULong) = check(Native.nativeMemberRelationshipClear(requireHandle(), observer.toLong(), target.toLong()))
    public fun setFactionMemberRelationship(faction: ULong, member: ULong, affinity: Float) = check(Native.nativeFactionMemberRelationshipSet(requireHandle(), faction.toLong(), member.toLong(), affinity))
    public fun clearFactionMemberRelationship(faction: ULong, member: ULong) = check(Native.nativeFactionMemberRelationshipClear(requireHandle(), faction.toLong(), member.toLong()))
    public fun setFactionRelationship(source: ULong, target: ULong, affinity: Float) = check(Native.nativeFactionRelationshipSet(requireHandle(), source.toLong(), target.toLong(), affinity))
    public fun clearFactionRelationship(source: ULong, target: ULong) = check(Native.nativeFactionRelationshipClear(requireHandle(), source.toLong(), target.toLong()))
    public fun memberRelationship(observer: ULong, target: ULong) = relationship { b -> Native.nativeMemberRelationshipGet(requireHandle(), observer.toLong(), target.toLong(), b) }
    public fun factionMemberRelationship(faction: ULong, member: ULong) = relationship { b -> Native.nativeFactionMemberRelationshipGet(requireHandle(), faction.toLong(), member.toLong(), b) }
    public fun factionRelationship(source: ULong, target: ULong) = relationship { b -> Native.nativeFactionRelationshipGet(requireHandle(), source.toLong(), target.toLong(), b) }
    public fun effectiveMemberRelationship(observer: ULong, target: ULong) = relationship { b -> Native.nativeEffectiveRelationshipGet(requireHandle(), observer.toLong(), target.toLong(), b) }

    public fun submit(deed: DirectWitnessDeed): SubmissionResult { val output = buffer(Layout.SUBMISSION); check(Native.nativeSubmit(requireHandle(), deed.deedId.toLong(), deed.observer.toLong(), deed.actor.toLong(), (deed.target ?: 0uL).toLong(), deed.impact, deed.aggression, deed.target != null, deed.threatensObserver, output)); return parseSubmission(output) }
    public fun submitBatch(deeds: List<DirectWitnessDeed>): List<SubmissionResult> { val input = buffer(Layout.DEED * deeds.size); deeds.forEach { encodeDeed(input, it) }; input.flip(); val output = buffer(Layout.SUBMISSION * deeds.size); val count = IntArray(1); val error = IntArray(1) { -1 }; check(Native.nativeSubmitBatch(requireHandle(), input, deeds.size, output, count, error), error[0].takeUnless { it == -1 }); return List(count[0]) { parseSubmission(output, it * Layout.SUBMISSION) } }
    public fun step(delta: ULong) = check(Native.nativeStep(requireHandle(), delta.toLong()))
    public fun advanceTo(tick: ULong) = check(Native.nativeAdvanceTo(requireHandle(), tick.toLong()))
    public fun memory(observer: ULong, deedId: ULong): MemoryRecord? { val output = buffer(Layout.MEMORY); val present = IntArray(1); check(Native.nativeMemoryGet(requireHandle(), observer.toLong(), deedId.toLong(), output, present)); return if (present[0] == 1) parseMemory(output) else null }
    public fun memories(observer: ULong): List<MemoryRecord> { val count = IntArray(1); check(Native.nativeMemoriesCount(requireHandle(), observer.toLong(), count)); if (count[0] == 0) return emptyList(); val output = buffer(Layout.MEMORY * count[0]); val written = IntArray(1); check(Native.nativeMemoriesRead(requireHandle(), observer.toLong(), output, count[0], written)); return List(written[0]) { parseMemory(output, it * Layout.MEMORY) } }
    public fun events(): List<SimulationEvent> { val count = IntArray(1); check(Native.nativeEventsCount(requireHandle(), count)); if (count[0] == 0) return emptyList(); val output = buffer(Layout.EVENT * count[0]); val written = IntArray(1); check(Native.nativeEventsRead(requireHandle(), output, count[0], written)); return List(written[0]) { parseEvent(output, it * Layout.EVENT) } }
    public fun clearEvents() = check(Native.nativeEventsClear(requireHandle()))

    private fun requireHandle(): Long = handle.takeUnless { it == 0L } ?: throw PersonaFluxException(ResultCode.INVALID_STATE, "simulation is closed")
    private fun buffer(size: Int): ByteBuffer = ByteBuffer.allocateDirect(size).order(ByteOrder.nativeOrder())
    private fun relationship(call: (ByteBuffer) -> Int): RelationshipLookup { val output = buffer(Layout.RELATIONSHIP); check(call(output)); return RelationshipLookup(output.get(8).toInt() != 0, output.getInt(12).toUInt(), output.getFloat(16)) }
    private fun parseSubmission(b: ByteBuffer, base: Int = 0): SubmissionResult { val kind = b.getInt(base + 8).toUInt(); val observer = b.getLong(base + 16).toULong(); val deedId = b.getLong(base + 24).toULong(); return if (kind == 1u) SubmissionResult.Applied(parseOutcome(b, base + 32)) else SubmissionResult.Duplicate(observer, deedId) }
    private fun parseOutcome(b: ByteBuffer, base: Int): DirectWitnessOutcome { val deedId = b.getLong(base + 8).toULong(); val observer = b.getLong(base + 16).toULong(); val actor = b.getLong(base + 24).toULong(); val target = if (b.get(base + 40).toInt() != 0) b.getLong(base + 32).toULong() else null; val relationship = RelationshipLookup(b.get(base + 44 + 8).toInt() != 0, b.getInt(base + 44 + 12).toUInt(), b.getFloat(base + 44 + 16)); val evaluation = parseEvaluation(b, base + 64); val previousAffinity = b.getFloat(base + 100); val currentAffinity = b.getFloat(base + 104); val previousPad = parsePad(b, base + 108); val currentPad = parsePad(b, base + 120); val memory = if (b.get(base + 132).toInt() != 0) parseMemory(b, base + 136) else null; return DirectWitnessOutcome(deedId, observer, actor, target, relationship, evaluation, previousAffinity, currentAffinity, previousPad, currentPad, memory) }
    private fun parseEvaluation(b: ByteBuffer, base: Int) = EvaluationResult(b.getFloat(base), b.getFloat(base + 4), b.getFloat(base + 8), parsePad(b, base + 12), b.getFloat(base + 24), b.getFloat(base + 28), b.getInt(base + 32).toUInt())
    private fun parsePad(b: ByteBuffer, base: Int) = Pad(b.getFloat(base), b.getFloat(base + 4), b.getFloat(base + 8))
    private fun parseMemory(b: ByteBuffer, base: Int = 0): MemoryRecord { val target = if (b.get(base + 40).toInt() != 0) b.getLong(base + 32).toULong() else null; return MemoryRecord(b.getLong(base + 8).toULong(), b.getLong(base + 16).toULong(), b.getLong(base + 24).toULong(), target, b.getFloat(base + 44), b.getFloat(base + 48), b.getFloat(base + 52), b.getInt(base + 56).toUInt(), b.getLong(base + 64).toULong(), b.getLong(base + 72).toULong()) }
    private fun parseEvent(b: ByteBuffer, base: Int = 0): SimulationEvent { val target = if (b.get(base + 64).toInt() != 0) b.getLong(base + 56).toULong() else null; return SimulationEvent(b.getInt(base + 8).toUInt(), b.getLong(base + 16).toULong(), b.getLong(base + 24).toULong(), b.getLong(base + 32).toULong(), b.getLong(base + 40).toULong(), b.getLong(base + 48).toULong(), target, b.getInt(base + 68).toUInt(), b.getLong(base + 72).toULong(), b.getLong(base + 80).toULong(), b.get(base + 88).toInt() != 0, b.get(base + 89).toInt() != 0, b.getFloat(base + 92), b.getFloat(base + 96), parseEvaluation(b, base + 100), parsePad(b, base + 136), parsePad(b, base + 148), b.getInt(base + 160).toUInt(), b.getInt(base + 164).toUInt(), b.getInt(base + 168).toUInt(), b.getFloat(base + 172), b.getLong(base + 176).toULong(), b.getLong(base + 184).toULong()) }
    private fun encodeDeed(b: ByteBuffer, deed: DirectWitnessDeed) { b.putInt(56); b.putInt(0); b.putLong(deed.deedId.toLong()); b.putLong(deed.observer.toLong()); b.putLong(deed.actor.toLong()); b.putLong((deed.target ?: 0uL).toLong()); b.putFloat(deed.impact); b.putFloat(deed.aggression); b.put(if (deed.target != null) 1 else 0); b.put(if (deed.threatensObserver) 1 else 0); b.putShort(0) }
}

internal object Layout { const val DEED = 56; const val RELATIONSHIP = 20; const val MEMORY = 80; const val SUBMISSION = 248; const val MEMBER_STATE = 32; const val EVENT = 192 }

private fun check(code: Int, errorIndex: Int? = null) {
    if (code == 0) return
    val message = Native.nativeLastError().toString(Charsets.UTF_8).ifEmpty { "PersonaFlux operation failed" }
    val result = ResultCode.values().firstOrNull { it.value == code } ?: ResultCode.INTERNAL_ERROR
    throw PersonaFluxException(result, message, errorIndex?.takeUnless { it < 0 }?.toUInt())
}
