using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
using PersonaFlux.Interop;

namespace PersonaFlux;

public sealed class Simulation : IDisposable
{
    public const uint AbiVersion = 0;
    public const uint ModelVersion = 1;
    private SafeSimulationHandle? _handle;

    public Simulation(ulong randomSeed)
    {
        Check(Native.pf_simulation_create(randomSeed, out var raw));
        _handle = new SafeSimulationHandle(raw);
    }

    public static uint ApiVersion => AbiVersion;
    public static uint EvaluationModelVersion => ModelVersion;
    public static uint QueryApiVersion() { Check(Native.pf_api_version(out var version)); return version; }
    public static uint QueryModelVersion() { Check(Native.pf_model_version(out var version)); return version; }

    public ulong RandomSeed { get { Ensure(); Check(Native.pf_simulation_random_seed(_handle!.DangerousGetHandle(), out var seed)); return seed; } }
    public ulong CurrentTick { get { Ensure(); Check(Native.pf_simulation_current_tick(_handle!.DangerousGetHandle(), out var tick)); return tick; } }

    public ulong AddFaction(string name)
    {
        Ensure(); if (name == null) throw new ArgumentNullException(nameof(name));
        var bytes = Encoding.UTF8.GetBytes(name); var ptr = AllocBytes(bytes);
        try { Check(Native.pf_faction_add(Handle, ptr, checked((uint)bytes.Length), out var id)); return id; }
        finally { Free(ptr); }
    }

    public string GetFactionName(ulong faction)
    {
        Ensure(); uint length; var sizeCode = Native.pf_faction_name_get(Handle, faction, IntPtr.Zero, 0, out length);
        if (sizeCode != (int)ResultCode.Ok && sizeCode != (int)ResultCode.BufferTooSmall) throw Failure(sizeCode, null);
        if (length == 0) return string.Empty;
        var ptr = Marshal.AllocHGlobal(checked((int)length));
        try { Check(Native.pf_faction_name_get(Handle, faction, ptr, length, out length)); var bytes = new byte[length]; Marshal.Copy(ptr, bytes, 0, checked((int)length)); return Encoding.UTF8.GetString(bytes); }
        finally { Free(ptr); }
    }

    public ulong AddMember(ulong faction) { Ensure(); Check(Native.pf_member_add(Handle, faction, out var member)); return member; }
    public MemberState GetMemberState(ulong member) { Ensure(); Check(Native.pf_member_state_get(Handle, member, out var state, (uint)Marshal.SizeOf<Native.MemberState>())); return new MemberState(state); }
    public float GetMemberAffinity(ulong observer, ulong actor) { Ensure(); Check(Native.pf_member_affinity_get(Handle, observer, actor, out var affinity)); return affinity; }

    public void SetMemberRelationship(ulong observer, ulong target, float affinity) { Ensure(); Check(Native.pf_relationship_member_to_member_set(Handle, observer, target, affinity)); }
    public void ClearMemberRelationship(ulong observer, ulong target) { Ensure(); Check(Native.pf_relationship_member_to_member_clear(Handle, observer, target)); }
    public void SetFactionMemberRelationship(ulong faction, ulong member, float affinity) { Ensure(); Check(Native.pf_relationship_faction_to_member_set(Handle, faction, member, affinity)); }
    public void ClearFactionMemberRelationship(ulong faction, ulong member) { Ensure(); Check(Native.pf_relationship_faction_to_member_clear(Handle, faction, member)); }
    public void SetFactionRelationship(ulong source, ulong target, float affinity) { Ensure(); Check(Native.pf_relationship_faction_to_faction_set(Handle, source, target, affinity)); }
    public void ClearFactionRelationship(ulong source, ulong target) { Ensure(); Check(Native.pf_relationship_faction_to_faction_clear(Handle, source, target)); }

    public RelationshipLookup GetMemberRelationship(ulong observer, ulong target) { Ensure(); Check(Native.pf_relationship_member_to_member_get(Handle, observer, target, out var value, (uint)Marshal.SizeOf<Native.RelationshipLookup>())); return DirectWitnessOutcome.ConvertRelationship(value); }
    public RelationshipLookup GetFactionMemberRelationship(ulong faction, ulong member) { Ensure(); Check(Native.pf_relationship_faction_to_member_get(Handle, faction, member, out var value, (uint)Marshal.SizeOf<Native.RelationshipLookup>())); return DirectWitnessOutcome.ConvertRelationship(value); }
    public RelationshipLookup GetFactionRelationship(ulong source, ulong target) { Ensure(); Check(Native.pf_relationship_faction_to_faction_get(Handle, source, target, out var value, (uint)Marshal.SizeOf<Native.RelationshipLookup>())); return DirectWitnessOutcome.ConvertRelationship(value); }
    public RelationshipLookup GetEffectiveMemberRelationship(ulong observer, ulong target) { Ensure(); Check(Native.pf_relationship_effective_member_get(Handle, observer, target, out var value, (uint)Marshal.SizeOf<Native.RelationshipLookup>())); return DirectWitnessOutcome.ConvertRelationship(value); }

    public SubmissionResult SubmitDirectWitness(DirectWitnessDeed deed)
    {
        Ensure(); var raw = ToNative(deed); Check(Native.pf_simulation_submit_direct_witness(Handle, ref raw, out var result, (uint)Marshal.SizeOf<Native.SubmissionResult>())); return new SubmissionResult(result);
    }

    public IReadOnlyList<SubmissionResult> SubmitDirectWitnessBatch(IReadOnlyList<DirectWitnessDeed> deeds)
    {
        Ensure(); if (deeds == null) throw new ArgumentNullException(nameof(deeds));
        var raw = new Native.DirectWitnessDeed[deeds.Count]; for (var i = 0; i < raw.Length; i++) raw[i] = ToNative(deeds[i]);
        var input = AllocArray(raw); var resultSize = Marshal.SizeOf<Native.SubmissionResult>(); var output = IntPtr.Zero;
        try
        {
            if (raw.Length > 0) output = Marshal.AllocHGlobal(checked(raw.Length * resultSize));
            var code = Native.pf_simulation_submit_direct_witness_batch(Handle, input, checked((uint)raw.Length), output, checked((uint)raw.Length), checked((uint)resultSize), out var count, out var errorIndex);
            if (code != (int)ResultCode.Ok) throw Failure(code, errorIndex == uint.MaxValue ? (uint?)null : errorIndex);
            var result = new List<SubmissionResult>(checked((int)count)); for (var i = 0; i < count; i++) result.Add(new SubmissionResult(Marshal.PtrToStructure<Native.SubmissionResult>(IntPtr.Add(output, checked(i * resultSize))))); return result;
        }
        finally { Free(input); Free(output); }
    }

    public void Step(ulong deltaTicks) { Ensure(); Check(Native.pf_simulation_step(Handle, deltaTicks)); }
    public void AdvanceTo(ulong targetTick) { Ensure(); Check(Native.pf_simulation_advance_to(Handle, targetTick)); }

    public MemoryRecord? GetMemory(ulong observer, ulong deedId)
    {
        Ensure(); Check(Native.pf_memory_get(Handle, observer, deedId, out var memory, (uint)Marshal.SizeOf<Native.MemoryRecord>(), out var present)); return present == 1 ? new MemoryRecord(memory) : (MemoryRecord?)null;
    }

    public IReadOnlyList<MemoryRecord> GetMemories(ulong observer)
    {
        Ensure(); Check(Native.pf_memories_count(Handle, observer, out var count)); if (count == 0) return Array.Empty<MemoryRecord>();
        var size = Marshal.SizeOf<Native.MemoryRecord>(); var output = Marshal.AllocHGlobal(checked((int)count * size));
        try { Check(Native.pf_memories_read(Handle, observer, output, count, checked((uint)size), out var written)); var list = new List<MemoryRecord>(checked((int)written)); for (var i = 0; i < written; i++) list.Add(new MemoryRecord(Marshal.PtrToStructure<Native.MemoryRecord>(IntPtr.Add(output, checked((int)i * size))))); return list; }
        finally { Free(output); }
    }

    public IReadOnlyList<SimulationEvent> GetEvents()
    {
        Ensure(); Check(Native.pf_events_count(Handle, out var count)); if (count == 0) return Array.Empty<SimulationEvent>();
        var size = Marshal.SizeOf<Native.Event>(); var output = Marshal.AllocHGlobal(checked((int)count * size));
        try { Check(Native.pf_events_read(Handle, output, count, checked((uint)size), out var written)); var list = new List<SimulationEvent>(checked((int)written)); for (var i = 0; i < written; i++) list.Add(new SimulationEvent(Marshal.PtrToStructure<Native.Event>(IntPtr.Add(output, checked((int)i * size))))); return list; }
        finally { Free(output); }
    }
    public void ClearEvents() { Ensure(); Check(Native.pf_events_clear(Handle)); }

    public void Dispose() { _handle?.Dispose(); _handle = null; GC.SuppressFinalize(this); }
    private IntPtr Handle { get { Ensure(); return _handle!.DangerousGetHandle(); } }
    private void Ensure() { if (_handle == null || _handle.IsInvalid || _handle.IsClosed) throw new ObjectDisposedException(nameof(Simulation)); }

    private static Native.DirectWitnessDeed ToNative(DirectWitnessDeed value) => new Native.DirectWitnessDeed { StructSize = (uint)Marshal.SizeOf<Native.DirectWitnessDeed>(), ApiVersion = AbiVersion, DeedId = value.DeedId, Observer = value.Observer, Actor = value.Actor, Target = value.Target ?? 0, Impact = value.Impact, Aggression = value.Aggression, HasTarget = value.Target.HasValue ? (byte)1 : (byte)0, ThreatensObserver = value.ThreatensObserver ? (byte)1 : (byte)0 };
    private static IntPtr AllocBytes(byte[] bytes) { if (bytes.Length == 0) return IntPtr.Zero; var ptr = Marshal.AllocHGlobal(bytes.Length); Marshal.Copy(bytes, 0, ptr, bytes.Length); return ptr; }
    private static IntPtr AllocArray<T>(T[] values) where T : struct { if (values.Length == 0) return IntPtr.Zero; var size = Marshal.SizeOf<T>(); var ptr = Marshal.AllocHGlobal(checked(values.Length * size)); for (var i = 0; i < values.Length; i++) Marshal.StructureToPtr(values[i], IntPtr.Add(ptr, checked(i * size)), false); return ptr; }
    private static void Free(IntPtr pointer) { if (pointer != IntPtr.Zero) Marshal.FreeHGlobal(pointer); }

    private static void Check(int code) { if (code != (int)ResultCode.Ok) throw Failure(code, null); }
    private static PersonaFluxException Failure(int code, uint? index) { return new PersonaFluxException((ResultCode)code, LastError(), index); }
    private static string LastError()
    {
        uint required; var scratch = new byte[1024]; var ptr = Marshal.AllocHGlobal(scratch.Length);
        try { var code = Native.pf_last_error_message_copy(ptr, (uint)scratch.Length, out required); if (code == (int)ResultCode.BufferTooSmall && required > scratch.Length) { Free(ptr); ptr = Marshal.AllocHGlobal(checked((int)required)); Native.pf_last_error_message_copy(ptr, required, out required); } if (required == 0) return "PersonaFlux operation failed"; var bytes = new byte[required]; Marshal.Copy(ptr, bytes, 0, checked((int)required)); return Encoding.UTF8.GetString(bytes); }
        finally { Free(ptr); }
    }
}
