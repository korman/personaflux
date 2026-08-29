#include <jni.h>
#include <cstdint>
#include <cstring>
#include <vector>

#include "personaflux.h"

namespace {

pf_simulation_t* simulation(jlong value) {
    return reinterpret_cast<pf_simulation_t*>(static_cast<uintptr_t>(value));
}

void put_long(JNIEnv* env, jlongArray output, uint64_t value) {
    if (output == nullptr || env->GetArrayLength(output) < 1) return;
    const jlong converted = static_cast<jlong>(value);
    env->SetLongArrayRegion(output, 0, 1, &converted);
}

void put_int(JNIEnv* env, jintArray output, uint32_t value) {
    if (output == nullptr || env->GetArrayLength(output) < 1) return;
    const jint converted = static_cast<jint>(value);
    env->SetIntArrayRegion(output, 0, 1, &converted);
}

void put_float(JNIEnv* env, jfloatArray output, float value) {
    if (output == nullptr || env->GetArrayLength(output) < 1) return;
    const jfloat converted = value;
    env->SetFloatArrayRegion(output, 0, 1, &converted);
}

void* direct(JNIEnv* env, jobject buffer, jlong required) {
    if (buffer == nullptr) return nullptr;
    if (env->GetDirectBufferCapacity(buffer) < required) return nullptr;
    return env->GetDirectBufferAddress(buffer);
}

pf_direct_witness_deed_t make_deed(jlong deed_id, jlong observer, jlong actor, jlong target,
                                   jfloat impact, jfloat aggression, jboolean has_target,
                                   jboolean threatens_observer) {
    pf_direct_witness_deed_t deed{};
    deed.struct_size = sizeof(deed);
    deed.api_version = PF_ABI_VERSION;
    deed.deed_id = static_cast<uint64_t>(deed_id);
    deed.observer = static_cast<uint64_t>(observer);
    deed.actor = static_cast<uint64_t>(actor);
    deed.target = static_cast<uint64_t>(target);
    deed.impact = impact;
    deed.aggression = aggression;
    deed.has_target = static_cast<uint8_t>(has_target ? 1 : 0);
    deed.threatens_observer = static_cast<uint8_t>(threatens_observer ? 1 : 0);
    return deed;
}

} // namespace

extern "C" JNIEXPORT jint JNICALL
Java_com_personaflux_Native_nativeApiVersion(JNIEnv* env, jclass, jintArray output) {
    uint32_t value = 0; const auto code = pf_api_version(&value); if (code == PF_OK) put_int(env, output, value); return code;
}

extern "C" JNIEXPORT jint JNICALL
Java_com_personaflux_Native_nativeModelVersion(JNIEnv* env, jclass, jintArray output) {
    uint32_t value = 0; const auto code = pf_model_version(&value); if (code == PF_OK) put_int(env, output, value); return code;
}

extern "C" JNIEXPORT jint JNICALL
Java_com_personaflux_Native_nativeCreate(JNIEnv* env, jclass, jlong seed, jlongArray output) {
    pf_simulation_t* result = nullptr;
    const auto code = pf_simulation_create(static_cast<uint64_t>(seed), &result);
    if (code == PF_OK) put_long(env, output, reinterpret_cast<uintptr_t>(result));
    return code;
}

extern "C" JNIEXPORT void JNICALL
Java_com_personaflux_Native_nativeDestroy(JNIEnv*, jclass, jlong handle) {
    pf_simulation_destroy(simulation(handle));
}

extern "C" JNIEXPORT jint JNICALL
Java_com_personaflux_Native_nativeRandomSeed(JNIEnv* env, jclass, jlong handle, jlongArray output) {
    uint64_t value = 0; const auto code = pf_simulation_random_seed(simulation(handle), &value);
    if (code == PF_OK) put_long(env, output, value); return code;
}

extern "C" JNIEXPORT jint JNICALL
Java_com_personaflux_Native_nativeAddFaction(JNIEnv* env, jclass, jlong handle, jbyteArray name, jlongArray output) {
    if (name == nullptr) return PF_INVALID_ARGUMENT;
    const auto length = env->GetArrayLength(name); std::vector<uint8_t> bytes(static_cast<size_t>(length));
    if (length > 0) env->GetByteArrayRegion(name, 0, length, reinterpret_cast<jbyte*>(bytes.data()));
    uint64_t faction = 0; const auto code = pf_faction_add(simulation(handle), bytes.data(), static_cast<uint32_t>(length), &faction);
    if (code == PF_OK) put_long(env, output, faction); return code;
}

extern "C" JNIEXPORT jbyteArray JNICALL
Java_com_personaflux_Native_nativeFactionName(JNIEnv* env, jclass, jlong handle, jlong faction, jintArray resultCode) {
    uint32_t length = 0; auto code = pf_faction_name_get(simulation(handle), static_cast<uint64_t>(faction), nullptr, 0, &length);
    if (code != PF_OK && code != PF_BUFFER_TOO_SMALL) { put_int(env, resultCode, static_cast<uint32_t>(code)); return nullptr; }
    std::vector<uint8_t> bytes(length); code = pf_faction_name_get(simulation(handle), static_cast<uint64_t>(faction), bytes.data(), length, &length);
    put_int(env, resultCode, static_cast<uint32_t>(code)); if (code != PF_OK) return nullptr;
    auto result = env->NewByteArray(static_cast<jsize>(length)); if (length > 0) env->SetByteArrayRegion(result, 0, static_cast<jsize>(length), reinterpret_cast<const jbyte*>(bytes.data())); return result;
}

extern "C" JNIEXPORT jint JNICALL
Java_com_personaflux_Native_nativeAddMember(JNIEnv* env, jclass, jlong handle, jlong faction, jlongArray output) {
    uint64_t member = 0; const auto code = pf_member_add(simulation(handle), static_cast<uint64_t>(faction), &member);
    if (code == PF_OK) put_long(env, output, member); return code;
}

extern "C" JNIEXPORT jint JNICALL
Java_com_personaflux_Native_nativeMemberState(JNIEnv* env, jclass, jlong handle, jlong member, jobject output) {
    auto* state = static_cast<pf_member_state_t*>(direct(env, output, sizeof(pf_member_state_t))); if (!state) return PF_INVALID_ARGUMENT;
    state->struct_size = sizeof(*state); state->api_version = PF_ABI_VERSION; return pf_member_state_get(simulation(handle), static_cast<uint64_t>(member), state, sizeof(*state));
}

extern "C" JNIEXPORT jint JNICALL
Java_com_personaflux_Native_nativeMemberAffinity(JNIEnv* env, jclass, jlong handle, jlong observer, jlong actor, jfloatArray output) {
    float value = 0; const auto code = pf_member_affinity_get(simulation(handle), static_cast<uint64_t>(observer), static_cast<uint64_t>(actor), &value); if (code == PF_OK) put_float(env, output, value); return code;
}

extern "C" JNIEXPORT jint JNICALL
Java_com_personaflux_Native_nativeMemberRelationshipSet(JNIEnv*, jclass, jlong handle, jlong observer, jlong target, jfloat affinity) { return pf_relationship_member_to_member_set(simulation(handle), static_cast<uint64_t>(observer), static_cast<uint64_t>(target), affinity); }
extern "C" JNIEXPORT jint JNICALL
Java_com_personaflux_Native_nativeMemberRelationshipClear(JNIEnv*, jclass, jlong handle, jlong observer, jlong target) { return pf_relationship_member_to_member_clear(simulation(handle), static_cast<uint64_t>(observer), static_cast<uint64_t>(target)); }
extern "C" JNIEXPORT jint JNICALL
Java_com_personaflux_Native_nativeFactionMemberRelationshipSet(JNIEnv*, jclass, jlong handle, jlong faction, jlong member, jfloat affinity) { return pf_relationship_faction_to_member_set(simulation(handle), static_cast<uint64_t>(faction), static_cast<uint64_t>(member), affinity); }
extern "C" JNIEXPORT jint JNICALL
Java_com_personaflux_Native_nativeFactionMemberRelationshipClear(JNIEnv*, jclass, jlong handle, jlong faction, jlong member) { return pf_relationship_faction_to_member_clear(simulation(handle), static_cast<uint64_t>(faction), static_cast<uint64_t>(member)); }
extern "C" JNIEXPORT jint JNICALL
Java_com_personaflux_Native_nativeFactionRelationshipSet(JNIEnv*, jclass, jlong handle, jlong source, jlong target, jfloat affinity) { return pf_relationship_faction_to_faction_set(simulation(handle), static_cast<uint64_t>(source), static_cast<uint64_t>(target), affinity); }
extern "C" JNIEXPORT jint JNICALL
Java_com_personaflux_Native_nativeFactionRelationshipClear(JNIEnv*, jclass, jlong handle, jlong source, jlong target) { return pf_relationship_faction_to_faction_clear(simulation(handle), static_cast<uint64_t>(source), static_cast<uint64_t>(target)); }

#define PF_REL_GETTER(name, function) \
extern "C" JNIEXPORT jint JNICALL Java_com_personaflux_Native_##name(JNIEnv* env, jclass, jlong handle, jlong first, jlong second, jobject output) { \
    auto* value = static_cast<pf_relationship_lookup_t*>(direct(env, output, sizeof(pf_relationship_lookup_t))); \
    if (!value) return PF_INVALID_ARGUMENT; value->struct_size = sizeof(*value); value->api_version = PF_ABI_VERSION; \
    return function(simulation(handle), static_cast<uint64_t>(first), static_cast<uint64_t>(second), value, sizeof(*value)); }

PF_REL_GETTER(nativeMemberRelationshipGet, pf_relationship_member_to_member_get)
PF_REL_GETTER(nativeFactionMemberRelationshipGet, pf_relationship_faction_to_member_get)
PF_REL_GETTER(nativeFactionRelationshipGet, pf_relationship_faction_to_faction_get)
PF_REL_GETTER(nativeEffectiveRelationshipGet, pf_relationship_effective_member_get)

#undef PF_REL_GETTER

extern "C" JNIEXPORT jint JNICALL
Java_com_personaflux_Native_nativeSubmit(JNIEnv* env, jclass, jlong handle, jlong deed_id, jlong observer, jlong actor, jlong target, jfloat impact, jfloat aggression, jboolean has_target, jboolean threatens_observer, jobject output) {
    auto* result = static_cast<pf_submission_result_t*>(direct(env, output, sizeof(pf_submission_result_t))); if (!result) return PF_INVALID_ARGUMENT;
    auto deed = make_deed(deed_id, observer, actor, target, impact, aggression, has_target, threatens_observer); result->struct_size = sizeof(*result); result->api_version = PF_ABI_VERSION;
    return pf_simulation_submit_direct_witness(simulation(handle), &deed, result, sizeof(*result));
}

extern "C" JNIEXPORT jint JNICALL
Java_com_personaflux_Native_nativeSubmitBatch(JNIEnv* env, jclass, jlong handle, jobject deeds, jint deed_count, jobject results, jintArray output_count, jintArray output_error_index) {
    if (output_count == nullptr || output_error_index == nullptr || env->GetArrayLength(output_count) < 1 || env->GetArrayLength(output_error_index) < 1) return PF_INVALID_ARGUMENT;
    auto* input = static_cast<pf_direct_witness_deed_t*>(direct(env, deeds, static_cast<jlong>(deed_count) * sizeof(pf_direct_witness_deed_t)));
    auto* output = static_cast<pf_submission_result_t*>(direct(env, results, static_cast<jlong>(deed_count) * sizeof(pf_submission_result_t)));
    if (deed_count < 0 || (deed_count > 0 && (!input || !output))) return PF_INVALID_ARGUMENT;
    auto* count = env->GetIntArrayElements(output_count, nullptr);
    auto* error = env->GetIntArrayElements(output_error_index, nullptr);
    if (!count || !error) {
        if (count) env->ReleaseIntArrayElements(output_count, count, JNI_ABORT);
        if (error) env->ReleaseIntArrayElements(output_error_index, error, JNI_ABORT);
        return PF_INTERNAL_ERROR;
    }
    const auto code = pf_simulation_submit_direct_witness_batch(simulation(handle), input, static_cast<uint32_t>(deed_count), output, static_cast<uint32_t>(deed_count), sizeof(pf_submission_result_t), reinterpret_cast<uint32_t*>(count), reinterpret_cast<uint32_t*>(error));
    env->ReleaseIntArrayElements(output_count, count, 0);
    env->ReleaseIntArrayElements(output_error_index, error, 0);
    return code;
}

extern "C" JNIEXPORT jint JNICALL
Java_com_personaflux_Native_nativeCurrentTick(JNIEnv* env, jclass, jlong handle, jlongArray output) { uint64_t tick = 0; const auto code = pf_simulation_current_tick(simulation(handle), &tick); if (code == PF_OK) put_long(env, output, tick); return code; }
extern "C" JNIEXPORT jint JNICALL
Java_com_personaflux_Native_nativeStep(JNIEnv*, jclass, jlong handle, jlong delta) { return pf_simulation_step(simulation(handle), static_cast<uint64_t>(delta)); }
extern "C" JNIEXPORT jint JNICALL
Java_com_personaflux_Native_nativeAdvanceTo(JNIEnv*, jclass, jlong handle, jlong tick) { return pf_simulation_advance_to(simulation(handle), static_cast<uint64_t>(tick)); }
extern "C" JNIEXPORT jint JNICALL
Java_com_personaflux_Native_nativeMemoryGet(JNIEnv* env, jclass, jlong handle, jlong observer, jlong deed_id, jobject output, jintArray present) { auto* record = static_cast<pf_memory_record_t*>(direct(env, output, sizeof(pf_memory_record_t))); if (!record) return PF_INVALID_ARGUMENT; uint8_t value = 0; const auto code = pf_memory_get(simulation(handle), static_cast<uint64_t>(observer), static_cast<uint64_t>(deed_id), record, sizeof(*record), &value); if (code == PF_OK) put_int(env, present, value); return code; }
extern "C" JNIEXPORT jint JNICALL
Java_com_personaflux_Native_nativeMemoriesCount(JNIEnv* env, jclass, jlong handle, jlong observer, jintArray output) { uint32_t count = 0; const auto code = pf_memories_count(simulation(handle), static_cast<uint64_t>(observer), &count); if (code == PF_OK) put_int(env, output, count); return code; }
extern "C" JNIEXPORT jint JNICALL
Java_com_personaflux_Native_nativeMemoriesRead(JNIEnv* env, jclass, jlong handle, jlong observer, jobject output, jint capacity, jintArray output_count) { auto* records = static_cast<pf_memory_record_t*>(direct(env, output, static_cast<jlong>(capacity) * sizeof(pf_memory_record_t))); if (capacity > 0 && !records) return PF_INVALID_ARGUMENT; uint32_t count = 0; const auto code = pf_memories_read(simulation(handle), static_cast<uint64_t>(observer), records, static_cast<uint32_t>(capacity), sizeof(pf_memory_record_t), &count); if (code == PF_OK) put_int(env, output_count, count); return code; }
extern "C" JNIEXPORT jint JNICALL
Java_com_personaflux_Native_nativeEventsCount(JNIEnv* env, jclass, jlong handle, jintArray output) { uint32_t count = 0; const auto code = pf_events_count(simulation(handle), &count); if (code == PF_OK) put_int(env, output, count); return code; }
extern "C" JNIEXPORT jint JNICALL
Java_com_personaflux_Native_nativeEventsRead(JNIEnv* env, jclass, jlong handle, jobject output, jint capacity, jintArray output_count) { auto* events = static_cast<pf_event_t*>(direct(env, output, static_cast<jlong>(capacity) * sizeof(pf_event_t))); if (capacity > 0 && !events) return PF_INVALID_ARGUMENT; uint32_t count = 0; const auto code = pf_events_read(simulation(handle), events, static_cast<uint32_t>(capacity), sizeof(pf_event_t), &count); if (code == PF_OK) put_int(env, output_count, count); return code; }
extern "C" JNIEXPORT jint JNICALL
Java_com_personaflux_Native_nativeEventsClear(JNIEnv*, jclass, jlong handle) { return pf_events_clear(simulation(handle)); }

extern "C" JNIEXPORT jbyteArray JNICALL
Java_com_personaflux_Native_nativeLastError(JNIEnv* env, jclass) {
    uint32_t length = 0; if (pf_last_error_message_copy(nullptr, 0, &length) != PF_OK && length == 0) return env->NewByteArray(0);
    std::vector<uint8_t> bytes(length); if (length > 0) pf_last_error_message_copy(bytes.data(), length, &length);
    auto result = env->NewByteArray(static_cast<jsize>(length)); if (length > 0) env->SetByteArrayRegion(result, 0, static_cast<jsize>(length), reinterpret_cast<const jbyte*>(bytes.data())); return result;
}
