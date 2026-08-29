#include "../include/personaflux.h"

#include <stddef.h>

_Static_assert(sizeof(pf_pad_t) == 12, "pf_pad_t layout changed");
_Static_assert(sizeof(pf_relationship_lookup_t) == 20, "relationship layout changed");
_Static_assert(sizeof(pf_evaluation_result_t) == 36, "evaluation layout changed");
_Static_assert(sizeof(pf_direct_witness_deed_t) == 56, "deed layout changed");
_Static_assert(sizeof(pf_memory_record_t) == 80, "memory layout changed");
_Static_assert(sizeof(pf_direct_witness_outcome_t) == 216, "outcome layout changed");
_Static_assert(sizeof(pf_submission_result_t) == 248, "submission layout changed");
_Static_assert(sizeof(pf_member_state_t) == 32, "member state layout changed");
_Static_assert(sizeof(pf_event_t) == 192, "event layout changed");
_Static_assert(offsetof(pf_direct_witness_deed_t, struct_size) == 0, "missing size prefix");
_Static_assert(offsetof(pf_direct_witness_deed_t, api_version) == 4, "missing version prefix");

int main(void) {
    pf_direct_witness_deed_t deed = {0};
    deed.struct_size = (uint32_t)sizeof(deed);
    deed.api_version = PF_ABI_VERSION;
    return (int)deed.api_version;
}
