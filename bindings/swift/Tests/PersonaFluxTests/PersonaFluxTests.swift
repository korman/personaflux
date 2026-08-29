import XCTest
@testable import PersonaFlux
import CPersonaFlux

final class PersonaFluxTests: XCTestCase {
    func testAbiLayout() {
        XCTAssertEqual(MemoryLayout<pf_pad_t>.size, 12)
        XCTAssertEqual(MemoryLayout<pf_relationship_lookup_t>.size, 20)
        XCTAssertEqual(MemoryLayout<pf_evaluation_result_t>.size, 36)
        XCTAssertEqual(MemoryLayout<pf_direct_witness_deed_t>.size, 56)
        XCTAssertEqual(MemoryLayout<pf_memory_record_t>.size, 80)
        XCTAssertEqual(MemoryLayout<pf_direct_witness_outcome_t>.size, 216)
        XCTAssertEqual(MemoryLayout<pf_submission_result_t>.size, 248)
        XCTAssertEqual(MemoryLayout<pf_member_state_t>.size, 32)
        XCTAssertEqual(MemoryLayout<pf_event_t>.size, 192)
    }

    func testCoreRoundTripThroughCAbi() throws {
        let simulation = try Simulation(randomSeed: 17)
        let faction = try simulation.addFaction(name: "observers")
        let observer = try simulation.addMember(to: faction)
        let actor = try simulation.addMember(to: faction)
        let target = try simulation.addMember(to: faction)
        try simulation.setMemberRelationship(observer: observer, target: target, affinity: 0.75)
        XCTAssertTrue(try simulation.effectiveMemberRelationship(observer: observer, target: target).present)

        let submission = try simulation.submit(DirectWitnessDeed(deedID: 42, observer: observer, actor: actor, target: target, impact: 0.8, aggression: 0.1, threatensObserver: false))
        if case .applied(let outcome) = submission {
            XCTAssertEqual(outcome.observer, observer)
            XCTAssertNotNil(outcome.memory)
        } else {
            XCTFail("expected an applied submission")
        }
        XCTAssertFalse(try simulation.events().isEmpty)
        XCTAssertNotNil(try simulation.memory(observer: observer, deedID: 42))
        try simulation.step(60)
        XCTAssertEqual(try simulation.currentTick, 60)
        simulation.close()
        simulation.close()
    }
}
