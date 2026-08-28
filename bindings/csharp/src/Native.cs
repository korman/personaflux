using System;
using System.Runtime.InteropServices;

namespace PersonaFlux.Interop;

internal static class Native
{
    internal const string LibraryName = "personaflux";

    [StructLayout(LayoutKind.Sequential)]
    internal struct Pad { public float Pleasure; public float Arousal; public float Dominance; }

    [StructLayout(LayoutKind.Sequential)]
    internal struct RelationshipLookup
    {
        public uint StructSize; public uint ApiVersion; public byte Present;
        public byte Reserved0; public byte Reserved1; public byte Reserved2;
        public uint Source; public float Affinity;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct EvaluationResult
    {
        public float Concern; public float EffectiveConfidence; public float RawAffinityDelta;
        public Pad RawPadDelta; public float EventIntensity; public float MemorySalience;
        public uint MemoryKind;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct DirectWitnessDeed
    {
        public uint StructSize; public uint ApiVersion; public ulong DeedId;
        public ulong Observer; public ulong Actor; public ulong Target;
        public float Impact; public float Aggression; public byte HasTarget;
        public byte ThreatensObserver; public byte Reserved0; public byte Reserved1;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct MemoryRecord
    {
        public uint StructSize; public uint ApiVersion; public ulong Observer;
        public ulong DeedId; public ulong Actor; public ulong Target; public byte HasTarget;
        public byte Reserved0; public byte Reserved1; public byte Reserved2;
        public float Impact; public float Aggression; public float Salience;
        public uint Kind; public ulong CreatedTick; public ulong ExpiresAt;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct DirectWitnessOutcome
    {
        public uint StructSize; public uint ApiVersion; public ulong DeedId;
        public ulong Observer; public ulong Actor; public ulong Target; public byte HasTarget;
        public byte ReservedTarget0; public byte ReservedTarget1; public byte ReservedTarget2;
        public RelationshipLookup Relationship; public EvaluationResult Evaluation;
        public float PreviousAffinity; public float CurrentAffinity;
        public Pad PreviousPad; public Pad CurrentPad; public byte HasMemory;
        public byte ReservedMemory0; public byte ReservedMemory1; public byte ReservedMemory2;
        public MemoryRecord Memory;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct SubmissionResult
    {
        public uint StructSize; public uint ApiVersion; public uint Kind; public uint Reserved;
        public ulong Observer; public ulong DeedId; public DirectWitnessOutcome Outcome;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct MemberState
    {
        public uint StructSize; public uint ApiVersion; public ulong FactionId; public Pad Pad;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct Event
    {
        public uint StructSize; public uint ApiVersion; public uint Kind; public uint Reserved;
        public ulong FactionId; public ulong MemberId; public ulong DeedId; public ulong Observer;
        public ulong Actor; public ulong Target; public byte HasTarget;
        public byte ReservedTarget0; public byte ReservedTarget1; public byte ReservedTarget2;
        public uint RelationshipLayer; public ulong RelationshipSourceId;
        public ulong RelationshipTargetId; public byte PreviousPresent; public byte CurrentPresent;
        public byte ReservedRelationship0; public byte ReservedRelationship1;
        public float PreviousAffinity; public float CurrentAffinity; public EvaluationResult Evaluation;
        public Pad PreviousPad; public Pad CurrentPad; public uint MemoryKind;
        public uint PreviousMemoryKind; public uint CurrentMemoryKind; public float Salience;
        public ulong PreviousTick; public ulong CurrentTick;
    }

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_api_version(out uint version);
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_model_version(out uint version);
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_simulation_create(ulong seed, out IntPtr simulation);
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void pf_simulation_destroy(IntPtr simulation);
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_simulation_random_seed(IntPtr simulation, out ulong seed);
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_faction_add(IntPtr simulation, IntPtr name, uint nameLen, out ulong faction);
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_faction_name_get(IntPtr simulation, ulong faction, IntPtr name, uint capacity, out uint nameLen);
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_member_add(IntPtr simulation, ulong faction, out ulong member);
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_member_state_get(IntPtr simulation, ulong member, out MemberState state, uint stateSize);
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_member_affinity_get(IntPtr simulation, ulong observer, ulong actor, out float affinity);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_relationship_member_to_member_set(IntPtr simulation, ulong observer, ulong target, float affinity);
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_relationship_member_to_member_clear(IntPtr simulation, ulong observer, ulong target);
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_relationship_faction_to_member_set(IntPtr simulation, ulong faction, ulong member, float affinity);
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_relationship_faction_to_member_clear(IntPtr simulation, ulong faction, ulong member);
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_relationship_faction_to_faction_set(IntPtr simulation, ulong source, ulong target, float affinity);
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_relationship_faction_to_faction_clear(IntPtr simulation, ulong source, ulong target);
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_relationship_member_to_member_get(IntPtr simulation, ulong observer, ulong target, out RelationshipLookup relationship, uint relationshipSize);
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_relationship_faction_to_member_get(IntPtr simulation, ulong faction, ulong member, out RelationshipLookup relationship, uint relationshipSize);
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_relationship_faction_to_faction_get(IntPtr simulation, ulong source, ulong target, out RelationshipLookup relationship, uint relationshipSize);
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_relationship_effective_member_get(IntPtr simulation, ulong observer, ulong target, out RelationshipLookup relationship, uint relationshipSize);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_simulation_submit_direct_witness(IntPtr simulation, ref DirectWitnessDeed deed, out SubmissionResult submission, uint submissionSize);
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_simulation_submit_direct_witness_batch(IntPtr simulation, IntPtr deeds, uint deedCount, IntPtr results, uint resultCapacity, uint resultElementSize, out uint resultCount, out uint errorIndex);
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_simulation_current_tick(IntPtr simulation, out ulong tick);
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_simulation_step(IntPtr simulation, ulong deltaTicks);
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_simulation_advance_to(IntPtr simulation, ulong targetTick);
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_memory_get(IntPtr simulation, ulong observer, ulong deedId, out MemoryRecord memory, uint memorySize, out byte present);
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_memories_count(IntPtr simulation, ulong observer, out uint count);
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_memories_read(IntPtr simulation, ulong observer, IntPtr records, uint capacity, uint elementSize, out uint count);
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_events_count(IntPtr simulation, out uint count);
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_events_read(IntPtr simulation, IntPtr events, uint capacity, uint elementSize, out uint count);
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_events_clear(IntPtr simulation);
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int pf_last_error_message_copy(IntPtr message, uint capacity, out uint length);
}
