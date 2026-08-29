#ifndef PERSONAFLUX_H
#define PERSONAFLUX_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define PF_ABI_VERSION ((uint32_t)0)
#define PF_MODEL_VERSION ((uint32_t)1)

typedef struct pf_simulation pf_simulation_t;
typedef uint64_t pf_faction_id_t;
typedef uint64_t pf_member_id_t;
typedef int32_t pf_result_t;

enum {
    PF_OK = 0,
    PF_INVALID_ARGUMENT = 1,
    PF_NOT_FOUND = 2,
    PF_INVALID_STATE = 3,
    PF_BUFFER_TOO_SMALL = 4,
    PF_SERIALIZATION_ERROR = 5,
    PF_VERSION_MISMATCH = 6,
    PF_INTERNAL_ERROR = 255
};

enum {
    PF_SUBMISSION_APPLIED = 1,
    PF_SUBMISSION_DUPLICATE = 2
};

enum {
    PF_RELATIONSHIP_MEMBER_TO_MEMBER = 1,
    PF_RELATIONSHIP_FACTION_TO_MEMBER = 2,
    PF_RELATIONSHIP_FACTION_TO_FACTION = 3
};

enum {
    PF_MEMORY_SHORT_TERM = 1,
    PF_MEMORY_LONG_TERM = 2
};

enum {
    PF_MEMORY_NONE = 0,
    PF_MEMORY_DECISION_SHORT_TERM = 1,
    PF_MEMORY_DECISION_LONG_TERM = 2
};

enum {
    PF_EVENT_FACTION_ADDED = 1,
    PF_EVENT_MEMBER_ADDED = 2,
    PF_EVENT_RELATIONSHIP_CHANGED = 3,
    PF_EVENT_DEED_EVALUATED = 4,
    PF_EVENT_AFFINITY_CHANGED = 5,
    PF_EVENT_PAD_CHANGED = 6,
    PF_EVENT_MEMORY_REMEMBERED = 7,
    PF_EVENT_MEMORY_UPGRADED = 8,
    PF_EVENT_MEMORY_EXPIRED = 9,
    PF_EVENT_TIME_ADVANCED = 10
};

typedef struct {
    float pleasure;
    float arousal;
    float dominance;
} pf_pad_t;

typedef struct {
    uint32_t struct_size;
    uint32_t api_version;
    uint8_t present;
    uint8_t reserved[3];
    uint32_t source;
    float affinity;
} pf_relationship_lookup_t;

typedef struct {
    float concern;
    float effective_confidence;
    float raw_affinity_delta;
    pf_pad_t raw_pad_delta;
    float event_intensity;
    float memory_salience;
    uint32_t memory_kind;
} pf_evaluation_result_t;

typedef struct {
    uint32_t struct_size;
    uint32_t api_version;
    uint64_t deed_id;
    uint64_t observer;
    uint64_t actor;
    uint64_t target;
    float impact;
    float aggression;
    uint8_t has_target;
    uint8_t threatens_observer;
    uint8_t reserved[2];
} pf_direct_witness_deed_t;

typedef struct {
    uint32_t struct_size;
    uint32_t api_version;
    uint64_t observer;
    uint64_t deed_id;
    uint64_t actor;
    uint64_t target;
    uint8_t has_target;
    uint8_t reserved[3];
    float impact;
    float aggression;
    float salience;
    uint32_t kind;
    uint64_t created_tick;
    uint64_t expires_at;
} pf_memory_record_t;

typedef struct {
    uint32_t struct_size;
    uint32_t api_version;
    uint64_t deed_id;
    uint64_t observer;
    uint64_t actor;
    uint64_t target;
    uint8_t has_target;
    uint8_t reserved_target[3];
    pf_relationship_lookup_t relationship;
    pf_evaluation_result_t evaluation;
    float previous_affinity;
    float current_affinity;
    pf_pad_t previous_pad;
    pf_pad_t current_pad;
    uint8_t has_memory;
    uint8_t reserved_memory[3];
    pf_memory_record_t memory;
} pf_direct_witness_outcome_t;

typedef struct {
    uint32_t struct_size;
    uint32_t api_version;
    uint32_t kind;
    uint32_t reserved;
    uint64_t observer;
    uint64_t deed_id;
    pf_direct_witness_outcome_t outcome;
} pf_submission_result_t;

typedef struct {
    uint32_t struct_size;
    uint32_t api_version;
    uint64_t faction_id;
    pf_pad_t pad;
} pf_member_state_t;

typedef struct {
    uint32_t struct_size;
    uint32_t api_version;
    uint32_t kind;
    uint32_t reserved;
    uint64_t faction_id;
    uint64_t member_id;
    uint64_t deed_id;
    uint64_t observer;
    uint64_t actor;
    uint64_t target;
    uint8_t has_target;
    uint8_t reserved_target[3];
    uint32_t relationship_layer;
    uint64_t relationship_source_id;
    uint64_t relationship_target_id;
    uint8_t previous_present;
    uint8_t current_present;
    uint8_t reserved_relationship[2];
    float previous_affinity;
    float current_affinity;
    pf_evaluation_result_t evaluation;
    pf_pad_t previous_pad;
    pf_pad_t current_pad;
    uint32_t memory_kind;
    uint32_t previous_memory_kind;
    uint32_t current_memory_kind;
    float salience;
    uint64_t previous_tick;
    uint64_t current_tick;
} pf_event_t;

pf_result_t pf_api_version(uint32_t *out_version);
pf_result_t pf_model_version(uint32_t *out_version);
pf_result_t pf_simulation_create(uint64_t random_seed, pf_simulation_t **out_simulation);
void pf_simulation_destroy(pf_simulation_t *simulation);
pf_result_t pf_simulation_random_seed(const pf_simulation_t *simulation, uint64_t *out_random_seed);

pf_result_t pf_faction_add(
    pf_simulation_t *simulation,
    const uint8_t *name,
    uint32_t name_len,
    uint64_t *out_faction_id);
pf_result_t pf_faction_name_get(
    const pf_simulation_t *simulation,
    uint64_t faction_id,
    uint8_t *out_name,
    uint32_t name_capacity,
    uint32_t *out_name_len);
pf_result_t pf_member_add(
    pf_simulation_t *simulation,
    uint64_t faction_id,
    uint64_t *out_member_id);
pf_result_t pf_member_state_get(
    const pf_simulation_t *simulation,
    uint64_t member_id,
    pf_member_state_t *out_state,
    uint32_t out_state_size);
pf_result_t pf_member_affinity_get(
    const pf_simulation_t *simulation,
    uint64_t observer,
    uint64_t actor,
    float *out_affinity);

pf_result_t pf_relationship_member_to_member_set(
    pf_simulation_t *simulation,
    uint64_t observer,
    uint64_t target,
    float affinity);
pf_result_t pf_relationship_member_to_member_clear(
    pf_simulation_t *simulation,
    uint64_t observer,
    uint64_t target);
pf_result_t pf_relationship_faction_to_member_set(
    pf_simulation_t *simulation,
    uint64_t faction,
    uint64_t member,
    float affinity);
pf_result_t pf_relationship_faction_to_member_clear(
    pf_simulation_t *simulation,
    uint64_t faction,
    uint64_t member);
pf_result_t pf_relationship_faction_to_faction_set(
    pf_simulation_t *simulation,
    uint64_t source,
    uint64_t target,
    float affinity);
pf_result_t pf_relationship_faction_to_faction_clear(
    pf_simulation_t *simulation,
    uint64_t source,
    uint64_t target);
pf_result_t pf_relationship_member_to_member_get(
    const pf_simulation_t *simulation,
    uint64_t observer,
    uint64_t target,
    pf_relationship_lookup_t *out_relationship,
    uint32_t out_relationship_size);
pf_result_t pf_relationship_faction_to_member_get(
    const pf_simulation_t *simulation,
    uint64_t faction,
    uint64_t member,
    pf_relationship_lookup_t *out_relationship,
    uint32_t out_relationship_size);
pf_result_t pf_relationship_faction_to_faction_get(
    const pf_simulation_t *simulation,
    uint64_t source,
    uint64_t target,
    pf_relationship_lookup_t *out_relationship,
    uint32_t out_relationship_size);
pf_result_t pf_relationship_effective_member_get(
    const pf_simulation_t *simulation,
    uint64_t observer,
    uint64_t target,
    pf_relationship_lookup_t *out_relationship,
    uint32_t out_relationship_size);

pf_result_t pf_simulation_submit_direct_witness(
    pf_simulation_t *simulation,
    const pf_direct_witness_deed_t *deed,
    pf_submission_result_t *out_submission,
    uint32_t out_submission_size);
pf_result_t pf_simulation_submit_direct_witness_batch(
    pf_simulation_t *simulation,
    const pf_direct_witness_deed_t *deeds,
    uint32_t deed_count,
    pf_submission_result_t *out_results,
    uint32_t result_capacity,
    uint32_t result_element_size,
    uint32_t *out_result_count,
    uint32_t *out_error_index);

pf_result_t pf_simulation_current_tick(const pf_simulation_t *simulation, uint64_t *out_tick);
pf_result_t pf_simulation_step(pf_simulation_t *simulation, uint64_t delta_ticks);
pf_result_t pf_simulation_advance_to(pf_simulation_t *simulation, uint64_t target_tick);

pf_result_t pf_memory_get(
    const pf_simulation_t *simulation,
    uint64_t observer,
    uint64_t deed_id,
    pf_memory_record_t *out_memory,
    uint32_t out_memory_size,
    uint8_t *out_present);
pf_result_t pf_memories_count(
    const pf_simulation_t *simulation,
    uint64_t observer,
    uint32_t *out_count);
pf_result_t pf_memories_read(
    const pf_simulation_t *simulation,
    uint64_t observer,
    pf_memory_record_t *out_records,
    uint32_t record_capacity,
    uint32_t record_element_size,
    uint32_t *out_count);

pf_result_t pf_events_count(const pf_simulation_t *simulation, uint32_t *out_count);
pf_result_t pf_events_read(
    const pf_simulation_t *simulation,
    pf_event_t *out_events,
    uint32_t event_capacity,
    uint32_t event_element_size,
    uint32_t *out_count);
pf_result_t pf_events_clear(pf_simulation_t *simulation);

pf_result_t pf_last_error_message_copy(
    uint8_t *out_message,
    uint32_t message_capacity,
    uint32_t *out_message_len);

#ifdef __cplusplus
}
#endif

#endif /* PERSONAFLUX_H */
