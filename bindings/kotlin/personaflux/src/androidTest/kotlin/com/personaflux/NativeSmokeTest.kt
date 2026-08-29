package com.personaflux

import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class NativeSmokeTest {
    @Test
    fun coreRoundTripThroughJni() {
        val simulation = Simulation(17uL)
        val faction = simulation.addFaction("observers")
        val observer = simulation.addMember(faction)
        val actor = simulation.addMember(faction)
        val target = simulation.addMember(faction)
        simulation.setMemberRelationship(observer, target, 0.75f)
        assertTrue(simulation.effectiveMemberRelationship(observer, target).present)
        val applied = simulation.submit(DirectWitnessDeed(42uL, observer, actor, target, 0.8f, 0.1f, false))
        assertTrue(applied is SubmissionResult.Applied)
        assertTrue(simulation.currentTick == 0uL)
        simulation.close()
        simulation.close()
    }
}
