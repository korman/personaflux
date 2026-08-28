package com.personaflux

internal object Native {
    init { System.loadLibrary("personaflux_jni") }

    external fun nativeApiVersion(output: IntArray): Int
    external fun nativeModelVersion(output: IntArray): Int
    external fun nativeCreate(seed: Long, output: LongArray): Int
    external fun nativeDestroy(handle: Long)
    external fun nativeRandomSeed(handle: Long, output: LongArray): Int
    external fun nativeAddFaction(handle: Long, name: ByteArray, output: LongArray): Int
    external fun nativeFactionName(handle: Long, faction: Long, resultCode: IntArray): ByteArray?
    external fun nativeAddMember(handle: Long, faction: Long, output: LongArray): Int
    external fun nativeMemberState(handle: Long, member: Long, output: java.nio.ByteBuffer): Int
    external fun nativeMemberAffinity(handle: Long, observer: Long, actor: Long, output: FloatArray): Int
    external fun nativeMemberRelationshipSet(handle: Long, observer: Long, target: Long, affinity: Float): Int
    external fun nativeMemberRelationshipClear(handle: Long, observer: Long, target: Long): Int
    external fun nativeFactionMemberRelationshipSet(handle: Long, faction: Long, member: Long, affinity: Float): Int
    external fun nativeFactionMemberRelationshipClear(handle: Long, faction: Long, member: Long): Int
    external fun nativeFactionRelationshipSet(handle: Long, source: Long, target: Long, affinity: Float): Int
    external fun nativeFactionRelationshipClear(handle: Long, source: Long, target: Long): Int
    external fun nativeMemberRelationshipGet(handle: Long, observer: Long, target: Long, output: java.nio.ByteBuffer): Int
    external fun nativeFactionMemberRelationshipGet(handle: Long, faction: Long, member: Long, output: java.nio.ByteBuffer): Int
    external fun nativeFactionRelationshipGet(handle: Long, source: Long, target: Long, output: java.nio.ByteBuffer): Int
    external fun nativeEffectiveRelationshipGet(handle: Long, observer: Long, target: Long, output: java.nio.ByteBuffer): Int
    external fun nativeSubmit(handle: Long, deedId: Long, observer: Long, actor: Long, target: Long, impact: Float, aggression: Float, hasTarget: Boolean, threatensObserver: Boolean, output: java.nio.ByteBuffer): Int
    external fun nativeSubmitBatch(handle: Long, deeds: java.nio.ByteBuffer, deedCount: Int, results: java.nio.ByteBuffer, resultCount: IntArray, errorIndex: IntArray): Int
    external fun nativeCurrentTick(handle: Long, output: LongArray): Int
    external fun nativeStep(handle: Long, delta: Long): Int
    external fun nativeAdvanceTo(handle: Long, tick: Long): Int
    external fun nativeMemoryGet(handle: Long, observer: Long, deedId: Long, output: java.nio.ByteBuffer, present: IntArray): Int
    external fun nativeMemoriesCount(handle: Long, observer: Long, output: IntArray): Int
    external fun nativeMemoriesRead(handle: Long, observer: Long, output: java.nio.ByteBuffer, capacity: Int, count: IntArray): Int
    external fun nativeEventsCount(handle: Long, output: IntArray): Int
    external fun nativeEventsRead(handle: Long, output: java.nio.ByteBuffer, capacity: Int, count: IntArray): Int
    external fun nativeEventsClear(handle: Long): Int
    external fun nativeLastError(): ByteArray
}
