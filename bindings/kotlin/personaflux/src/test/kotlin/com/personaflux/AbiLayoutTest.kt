package com.personaflux

import org.junit.Assert.assertEquals
import org.junit.Test

class AbiLayoutTest {
    @Test
    fun v0SizesAreExplicit() {
        assertEquals(56, Layout.DEED)
        assertEquals(20, Layout.RELATIONSHIP)
        assertEquals(80, Layout.MEMORY)
        assertEquals(248, Layout.SUBMISSION)
        assertEquals(32, Layout.MEMBER_STATE)
        assertEquals(192, Layout.EVENT)
    }
}
