use std::mem::size_of;
use std::ptr;

use personaflux::{
    PF_ABI_VERSION, PF_BUFFER_TOO_SMALL, PF_EVENT_DEED_EVALUATED, PF_EVENT_FACTION_ADDED,
    PF_EVENT_MEMBER_ADDED, PF_EVENT_TIME_ADVANCED, PF_INVALID_ARGUMENT, PF_MEMORY_SHORT_TERM,
    PF_NOT_FOUND, PF_OK, PF_RELATIONSHIP_MEMBER_TO_MEMBER, PF_SUBMISSION_APPLIED,
    PF_SUBMISSION_DUPLICATE, PF_VERSION_MISMATCH, PfDirectWitnessDeed, PfEvent, PfMemberState,
    PfMemoryRecord, PfRelationshipLookup, PfSimulation, PfSubmissionResult, pf_api_version,
    pf_events_clear, pf_events_count, pf_events_read, pf_faction_add, pf_faction_name_get,
    pf_last_error_message_copy, pf_member_add, pf_member_affinity_get, pf_member_state_get,
    pf_memories_count, pf_memories_read, pf_memory_get, pf_model_version,
    pf_relationship_effective_member_get, pf_relationship_member_to_member_get,
    pf_relationship_member_to_member_set, pf_simulation_create, pf_simulation_current_tick,
    pf_simulation_destroy, pf_simulation_step, pf_simulation_submit_direct_witness,
    pf_simulation_submit_direct_witness_batch,
};

fn deed(
    deed_id: u64,
    observer: u64,
    actor: u64,
    target: Option<u64>,
    impact: f32,
    aggression: f32,
) -> PfDirectWitnessDeed {
    PfDirectWitnessDeed {
        struct_size: size_of::<PfDirectWitnessDeed>() as u32,
        api_version: PF_ABI_VERSION,
        deed_id,
        observer,
        actor,
        target: target.unwrap_or(0),
        impact,
        aggression,
        has_target: u8::from(target.is_some()),
        threatens_observer: 0,
        reserved: [0; 2],
    }
}

unsafe fn create() -> *mut PfSimulation {
    let mut simulation = ptr::null_mut();
    // SAFETY: The output pointer points to local writable storage.
    assert_eq!(unsafe { pf_simulation_create(17, &mut simulation) }, PF_OK);
    simulation
}

unsafe fn add_faction(simulation: *mut PfSimulation, name: &str) -> u64 {
    let mut faction = 0;
    // SAFETY: The string pointer and output pointer remain valid for this call.
    assert_eq!(
        unsafe { pf_faction_add(simulation, name.as_ptr(), name.len() as u32, &mut faction,) },
        PF_OK
    );
    faction
}

unsafe fn add_member(simulation: *mut PfSimulation, faction: u64) -> u64 {
    let mut member = 0;
    // SAFETY: The output pointer points to local writable storage.
    assert_eq!(
        unsafe { pf_member_add(simulation, faction, &mut member) },
        PF_OK
    );
    member
}

#[test]
fn exposes_versions_and_lifecycle() {
    let mut abi = 99;
    let mut model = 99;
    // SAFETY: Version output pointers point to local writable storage.
    unsafe {
        assert_eq!(pf_api_version(&mut abi), PF_OK);
        assert_eq!(pf_model_version(&mut model), PF_OK);
    }
    assert_eq!(abi, PF_ABI_VERSION);
    assert_eq!(model, 1);

    // SAFETY: Null destroy is explicitly a no-op.
    unsafe { pf_simulation_destroy(ptr::null_mut()) };
    // SAFETY: Lifecycle helper owns and destroys the live handle.
    unsafe {
        let simulation = create();
        pf_simulation_destroy(simulation);
    }
}

#[test]
fn wire_layout_has_fixed_v0_sizes() {
    assert_eq!(size_of::<personaflux::PfPad>(), 12);
    assert_eq!(size_of::<PfRelationshipLookup>(), 20);
    assert_eq!(size_of::<personaflux::PfEvaluationResult>(), 36);
    assert_eq!(size_of::<PfDirectWitnessDeed>(), 56);
    assert_eq!(size_of::<PfMemoryRecord>(), 80);
    assert_eq!(size_of::<personaflux::PfDirectWitnessOutcome>(), 216);
    assert_eq!(size_of::<PfSubmissionResult>(), 248);
    assert_eq!(size_of::<PfMemberState>(), 32);
    assert_eq!(size_of::<PfEvent>(), 192);
}

#[test]
fn runs_entities_relationship_deed_memory_time_and_events() {
    // SAFETY: All handles and pointers are valid for the duration of each call.
    unsafe {
        let simulation = create();
        let faction = add_faction(simulation, "observers");
        let observer = add_member(simulation, faction);
        let actor = add_member(simulation, faction);
        let target = add_member(simulation, faction);

        let mut name = [0u8; 9];
        let mut name_len = 0;
        assert_eq!(
            pf_faction_name_get(
                simulation,
                faction,
                name.as_mut_ptr(),
                name.len() as u32,
                &mut name_len,
            ),
            PF_OK
        );
        assert_eq!(&name[..name_len as usize], b"observers");

        let mut events_count = 0;
        assert_eq!(pf_events_count(simulation, &mut events_count), PF_OK);
        assert_eq!(events_count, 4);
        let mut events = vec![PfEvent::default(); events_count as usize];
        assert_eq!(
            pf_events_read(
                simulation,
                events.as_mut_ptr(),
                events_count,
                size_of::<PfEvent>() as u32,
                &mut events_count,
            ),
            PF_OK
        );
        assert_eq!(events[0].kind, PF_EVENT_FACTION_ADDED);
        assert_eq!(events[1].kind, PF_EVENT_MEMBER_ADDED);
        assert_eq!(events[3].kind, PF_EVENT_MEMBER_ADDED);
        let mut unread_count = 0;
        assert_eq!(pf_events_count(simulation, &mut unread_count), PF_OK);
        assert_eq!(unread_count, 4);
        assert_eq!(pf_events_clear(simulation), PF_OK);

        assert_eq!(
            pf_relationship_member_to_member_set(simulation, observer, target, 0.75),
            PF_OK
        );
        let mut relationship = PfRelationshipLookup::default();
        assert_eq!(
            pf_relationship_member_to_member_get(
                simulation,
                observer,
                target,
                &mut relationship,
                size_of::<PfRelationshipLookup>() as u32,
            ),
            PF_OK
        );
        assert_eq!(relationship.present, 1);
        assert_eq!(relationship.source, PF_RELATIONSHIP_MEMBER_TO_MEMBER);
        assert!((relationship.affinity - 0.75).abs() < f32::EPSILON);
        let mut effective = PfRelationshipLookup::default();
        assert_eq!(
            pf_relationship_effective_member_get(
                simulation,
                observer,
                target,
                &mut effective,
                size_of::<PfRelationshipLookup>() as u32,
            ),
            PF_OK
        );
        assert_eq!(effective.source, PF_RELATIONSHIP_MEMBER_TO_MEMBER);

        let mut deed_result = PfSubmissionResult::default();
        let deed = deed(41, observer, actor, Some(target), 0.5, 0.0);
        assert_eq!(
            pf_simulation_submit_direct_witness(
                simulation,
                &deed,
                &mut deed_result,
                size_of::<PfSubmissionResult>() as u32,
            ),
            PF_OK
        );
        assert_eq!(deed_result.kind, PF_SUBMISSION_APPLIED);
        assert_eq!(deed_result.outcome.has_memory, 1);
        assert_eq!(deed_result.outcome.memory.kind, PF_MEMORY_SHORT_TERM);

        let mut affinity = 0.0;
        assert_eq!(
            pf_member_affinity_get(simulation, observer, actor, &mut affinity),
            PF_OK
        );
        assert!(affinity > 0.0);
        let mut state = PfMemberState::default();
        assert_eq!(
            pf_member_state_get(
                simulation,
                observer,
                &mut state,
                size_of::<PfMemberState>() as u32,
            ),
            PF_OK
        );
        assert_eq!(state.faction_id, faction);

        let mut memory = PfMemoryRecord::default();
        let mut present = 0;
        assert_eq!(
            pf_memory_get(
                simulation,
                observer,
                41,
                &mut memory,
                size_of::<PfMemoryRecord>() as u32,
                &mut present,
            ),
            PF_OK
        );
        assert_eq!(present, 1);
        let mut memory_count = 0;
        assert_eq!(
            pf_memories_count(simulation, observer, &mut memory_count),
            PF_OK
        );
        assert_eq!(memory_count, 1);
        let mut memories = vec![PfMemoryRecord::default(); 1];
        assert_eq!(
            pf_memories_read(
                simulation,
                observer,
                memories.as_mut_ptr(),
                1,
                size_of::<PfMemoryRecord>() as u32,
                &mut memory_count,
            ),
            PF_OK
        );
        assert_eq!(memory_count, 1);

        let mut tick = 99;
        assert_eq!(pf_simulation_current_tick(simulation, &mut tick), PF_OK);
        assert_eq!(tick, 0);
        assert_eq!(pf_simulation_step(simulation, 60), PF_OK);
        assert_eq!(pf_simulation_current_tick(simulation, &mut tick), PF_OK);
        assert_eq!(tick, 60);

        let mut all_events_count = 0;
        assert_eq!(pf_events_count(simulation, &mut all_events_count), PF_OK);
        let mut all_events = vec![PfEvent::default(); all_events_count as usize];
        assert_eq!(
            pf_events_read(
                simulation,
                all_events.as_mut_ptr(),
                all_events_count,
                size_of::<PfEvent>() as u32,
                &mut all_events_count,
            ),
            PF_OK
        );
        assert!(
            all_events
                .iter()
                .any(|event| event.kind == PF_EVENT_DEED_EVALUATED)
        );
        assert!(
            all_events
                .iter()
                .any(|event| event.kind == PF_EVENT_TIME_ADVANCED)
        );
        assert_eq!(pf_events_clear(simulation), PF_OK);
        pf_simulation_destroy(simulation);
    }
}

#[test]
fn batch_is_idempotent_and_atomic_and_reports_buffer_errors() {
    // SAFETY: All handles and pointers are valid for the duration of each call.
    unsafe {
        let simulation = create();
        let faction = add_faction(simulation, "f");
        let observer = add_member(simulation, faction);
        let actor = add_member(simulation, faction);
        assert_eq!(pf_events_clear(simulation), PF_OK);

        let deeds = [
            deed(7, observer, actor, None, 0.4, 0.0),
            deed(7, observer, actor, None, -1.0, 1.0),
        ];
        let mut results = vec![PfSubmissionResult::default(); 2];
        let mut result_count = 0;
        let mut error_index = 0;
        assert_eq!(
            pf_simulation_submit_direct_witness_batch(
                simulation,
                deeds.as_ptr(),
                deeds.len() as u32,
                results.as_mut_ptr(),
                results.len() as u32,
                size_of::<PfSubmissionResult>() as u32,
                &mut result_count,
                &mut error_index,
            ),
            PF_OK
        );
        assert_eq!(result_count, 2);
        assert_eq!(results[0].kind, PF_SUBMISSION_APPLIED);
        assert_eq!(results[1].kind, PF_SUBMISSION_DUPLICATE);

        let mut too_small_count = 0;
        assert_eq!(
            pf_simulation_submit_direct_witness_batch(
                simulation,
                deeds.as_ptr(),
                deeds.len() as u32,
                ptr::null_mut(),
                0,
                size_of::<PfSubmissionResult>() as u32,
                &mut too_small_count,
                &mut error_index,
            ),
            PF_BUFFER_TOO_SMALL
        );
        assert_eq!(too_small_count, 2);

        let invalid = deed(8, observer, actor, Some(actor), 0.5, 0.0);
        let mut invalid_result = vec![PfSubmissionResult::default(); 1];
        assert_eq!(
            pf_simulation_submit_direct_witness_batch(
                simulation,
                &invalid,
                1,
                invalid_result.as_mut_ptr(),
                1,
                size_of::<PfSubmissionResult>() as u32,
                &mut result_count,
                &mut error_index,
            ),
            PF_INVALID_ARGUMENT
        );
        assert_eq!(error_index, 0);
        assert_eq!(result_count, 0);

        let mut affinity = 0.0;
        assert_eq!(
            pf_member_affinity_get(simulation, observer, actor, &mut affinity),
            PF_OK
        );
        assert!(affinity > 0.0);
        pf_simulation_destroy(simulation);
    }
}

#[test]
fn errors_are_mapped_and_last_error_is_copyable() {
    // SAFETY: Null pointers are intentional validation cases.
    unsafe {
        assert_eq!(
            pf_member_add(ptr::null_mut(), 1, ptr::null_mut()),
            PF_INVALID_ARGUMENT
        );
        let simulation = create();
        let faction = add_faction(simulation, "f");
        assert_eq!(pf_events_clear(simulation), PF_OK);
        assert_eq!(
            pf_faction_add(simulation, b"ignored".as_ptr(), 7, ptr::null_mut()),
            PF_INVALID_ARGUMENT
        );
        let mut event_count = 0;
        assert_eq!(pf_events_count(simulation, &mut event_count), PF_OK);
        assert_eq!(event_count, 0);

        let observer = add_member(simulation, faction);
        let actor = add_member(simulation, faction);
        assert_eq!(pf_events_clear(simulation), PF_OK);
        let deed = deed(9, observer, actor, None, 0.5, 0.0);
        assert_eq!(
            pf_simulation_submit_direct_witness(
                simulation,
                &deed,
                ptr::null_mut(),
                size_of::<PfSubmissionResult>() as u32,
            ),
            PF_INVALID_ARGUMENT
        );
        let mut affinity = 0.0;
        assert_eq!(
            pf_member_affinity_get(simulation, observer, actor, &mut affinity),
            PF_OK
        );
        assert_eq!(affinity, 0.0);
        assert_eq!(pf_events_count(simulation, &mut event_count), PF_OK);
        assert_eq!(event_count, 0);

        let mut state = PfMemberState::default();
        assert_eq!(
            pf_member_state_get(
                simulation,
                999,
                &mut state,
                size_of::<PfMemberState>() as u32,
            ),
            PF_NOT_FOUND
        );
        let mut length = 0;
        assert_eq!(
            pf_last_error_message_copy(ptr::null_mut(), 0, &mut length),
            PF_BUFFER_TOO_SMALL
        );
        assert!(length > 0);
        let mut message = vec![0u8; length as usize];
        assert_eq!(
            pf_last_error_message_copy(message.as_mut_ptr(), message.len() as u32, &mut length),
            PF_OK
        );
        assert!(!message.is_empty());
        pf_simulation_destroy(simulation);
    }
}

#[test]
fn rejects_invalid_wire_values_without_side_effects() {
    // SAFETY: All handles and pointers are valid for the duration of each call.
    unsafe {
        let simulation = create();
        let faction = add_faction(simulation, "f");
        let observer = add_member(simulation, faction);
        let actor = add_member(simulation, faction);
        assert_eq!(pf_events_clear(simulation), PF_OK);

        assert_eq!(
            pf_relationship_member_to_member_set(simulation, observer, actor, f32::NAN),
            PF_INVALID_ARGUMENT
        );
        let mut count = 0;
        assert_eq!(pf_events_count(simulation, &mut count), PF_OK);
        assert_eq!(count, 0);

        let mut result = PfSubmissionResult::default();
        let mut invalid_deed = deed(1, observer, actor, None, f32::INFINITY, 0.0);
        assert_eq!(
            pf_simulation_submit_direct_witness(
                simulation,
                &invalid_deed,
                &mut result,
                size_of::<PfSubmissionResult>() as u32,
            ),
            PF_INVALID_ARGUMENT
        );
        invalid_deed.api_version = PF_ABI_VERSION + 1;
        assert_eq!(
            pf_simulation_submit_direct_witness(
                simulation,
                &invalid_deed,
                &mut result,
                size_of::<PfSubmissionResult>() as u32,
            ),
            PF_VERSION_MISMATCH
        );
        assert_eq!(pf_events_count(simulation, &mut count), PF_OK);
        assert_eq!(count, 0);
        pf_simulation_destroy(simulation);
    }
}
