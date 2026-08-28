using System;

namespace PersonaFlux;

public enum ResultCode : int
{
    Ok = 0, InvalidArgument = 1, NotFound = 2, InvalidState = 3,
    BufferTooSmall = 4, SerializationError = 5, VersionMismatch = 6, InternalError = 255
}

public enum SubmissionKind : uint { Applied = 1, Duplicate = 2 }
public enum MemoryKind : uint { ShortTerm = 1, LongTerm = 2 }
public enum RelationshipSource : uint { MemberToMember = 1, FactionToMember = 2, FactionToFaction = 3 }
public enum EventKind : uint
{
    FactionAdded = 1, MemberAdded = 2, RelationshipChanged = 3, DeedEvaluated = 4,
    AffinityChanged = 5, PadChanged = 6, MemoryRemembered = 7, MemoryUpgraded = 8,
    MemoryExpired = 9, TimeAdvanced = 10
}

public sealed class PersonaFluxException : Exception
{
    public ResultCode Code { get; }
    public uint? ErrorIndex { get; }

    internal PersonaFluxException(ResultCode code, string message, uint? errorIndex = null)
        : base(message) { Code = code; ErrorIndex = errorIndex; }
}

public readonly struct Pad
{
    public float Pleasure { get; }
    public float Arousal { get; }
    public float Dominance { get; }
    public Pad(float pleasure, float arousal, float dominance)
        { Pleasure = pleasure; Arousal = arousal; Dominance = dominance; }
}

public readonly struct RelationshipLookup
{
    public bool Present { get; }
    public RelationshipSource Source { get; }
    public float Affinity { get; }
    public RelationshipLookup(bool present, RelationshipSource source, float affinity)
        { Present = present; Source = source; Affinity = affinity; }
}

public readonly struct EvaluationResult
{
    public float Concern { get; }
    public float EffectiveConfidence { get; }
    public float RawAffinityDelta { get; }
    public Pad RawPadDelta { get; }
    public float EventIntensity { get; }
    public float MemorySalience { get; }
    public uint MemoryDecision { get; }
    public EvaluationResult(float concern, float confidence, float affinityDelta, Pad padDelta, float intensity, float salience, uint decision)
    { Concern = concern; EffectiveConfidence = confidence; RawAffinityDelta = affinityDelta; RawPadDelta = padDelta; EventIntensity = intensity; MemorySalience = salience; MemoryDecision = decision; }
}

public readonly struct DirectWitnessDeed
{
    public ulong DeedId { get; }
    public ulong Observer { get; }
    public ulong Actor { get; }
    public ulong? Target { get; }
    public float Impact { get; }
    public float Aggression { get; }
    public bool ThreatensObserver { get; }
    public DirectWitnessDeed(ulong deedId, ulong observer, ulong actor, ulong? target, float impact, float aggression, bool threatensObserver)
    { DeedId = deedId; Observer = observer; Actor = actor; Target = target; Impact = impact; Aggression = aggression; ThreatensObserver = threatensObserver; }
}

public readonly struct MemoryRecord
{
    public ulong Observer { get; }
    public ulong DeedId { get; }
    public ulong Actor { get; }
    public ulong? Target { get; }
    public float Impact { get; }
    public float Aggression { get; }
    public float Salience { get; }
    public MemoryKind Kind { get; }
    public ulong CreatedTick { get; }
    public ulong ExpiresAt { get; }
    internal MemoryRecord(Interop.Native.MemoryRecord value)
    { Observer = value.Observer; DeedId = value.DeedId; Actor = value.Actor; Target = value.HasTarget == 1 ? value.Target : (ulong?)null; Impact = value.Impact; Aggression = value.Aggression; Salience = value.Salience; Kind = (MemoryKind)value.Kind; CreatedTick = value.CreatedTick; ExpiresAt = value.ExpiresAt; }
}

public readonly struct MemberState
{
    public ulong FactionId { get; }
    public Pad Pad { get; }
    internal MemberState(Interop.Native.MemberState value) { FactionId = value.FactionId; Pad = new Pad(value.Pad.Pleasure, value.Pad.Arousal, value.Pad.Dominance); }
}

public readonly struct DirectWitnessOutcome
{
    public ulong DeedId { get; }
    public ulong Observer { get; }
    public ulong Actor { get; }
    public ulong? Target { get; }
    public RelationshipLookup Relationship { get; }
    public EvaluationResult Evaluation { get; }
    public float PreviousAffinity { get; }
    public float CurrentAffinity { get; }
    public Pad PreviousPad { get; }
    public Pad CurrentPad { get; }
    public MemoryRecord? Memory { get; }
    internal DirectWitnessOutcome(Interop.Native.DirectWitnessOutcome value)
    {
        DeedId = value.DeedId; Observer = value.Observer; Actor = value.Actor;
        Target = value.HasTarget == 1 ? value.Target : (ulong?)null;
        Relationship = ConvertRelationship(value.Relationship);
        Evaluation = ConvertEvaluation(value.Evaluation);
        PreviousAffinity = value.PreviousAffinity; CurrentAffinity = value.CurrentAffinity;
        PreviousPad = ConvertPad(value.PreviousPad); CurrentPad = ConvertPad(value.CurrentPad);
        Memory = value.HasMemory == 1 ? new MemoryRecord(value.Memory) : (MemoryRecord?)null;
    }
    internal static Pad ConvertPad(Interop.Native.Pad value) => new Pad(value.Pleasure, value.Arousal, value.Dominance);
    internal static RelationshipLookup ConvertRelationship(Interop.Native.RelationshipLookup value) => new RelationshipLookup(value.Present == 1, (RelationshipSource)value.Source, value.Affinity);
    internal static EvaluationResult ConvertEvaluation(Interop.Native.EvaluationResult value) => new EvaluationResult(value.Concern, value.EffectiveConfidence, value.RawAffinityDelta, ConvertPad(value.RawPadDelta), value.EventIntensity, value.MemorySalience, value.MemoryKind);
}

public readonly struct SubmissionResult
{
    public SubmissionKind Kind { get; }
    public ulong Observer { get; }
    public ulong DeedId { get; }
    public DirectWitnessOutcome? Outcome { get; }
    internal SubmissionResult(Interop.Native.SubmissionResult value)
    { Kind = (SubmissionKind)value.Kind; Observer = value.Observer; DeedId = value.DeedId; Outcome = Kind == SubmissionKind.Applied ? new DirectWitnessOutcome(value.Outcome) : (DirectWitnessOutcome?)null; }
}

public readonly struct SimulationEvent
{
    public EventKind Kind { get; }
    public ulong FactionId { get; }
    public ulong MemberId { get; }
    public ulong DeedId { get; }
    public ulong Observer { get; }
    public ulong Actor { get; }
    public ulong? Target { get; }
    public RelationshipSource RelationshipLayer { get; }
    public ulong RelationshipSourceId { get; }
    public ulong RelationshipTargetId { get; }
    public bool PreviousPresent { get; }
    public bool CurrentPresent { get; }
    public float PreviousAffinity { get; }
    public float CurrentAffinity { get; }
    public EvaluationResult Evaluation { get; }
    public Pad PreviousPad { get; }
    public Pad CurrentPad { get; }
    public MemoryKind MemoryKind { get; }
    public MemoryKind PreviousMemoryKind { get; }
    public MemoryKind CurrentMemoryKind { get; }
    public float Salience { get; }
    public ulong PreviousTick { get; }
    public ulong CurrentTick { get; }
    internal SimulationEvent(Interop.Native.Event value)
    {
        Kind = (EventKind)value.Kind; FactionId = value.FactionId; MemberId = value.MemberId;
        DeedId = value.DeedId; Observer = value.Observer; Actor = value.Actor;
        Target = value.HasTarget == 1 ? value.Target : (ulong?)null;
        RelationshipLayer = (RelationshipSource)value.RelationshipLayer;
        RelationshipSourceId = value.RelationshipSourceId; RelationshipTargetId = value.RelationshipTargetId;
        PreviousPresent = value.PreviousPresent == 1; CurrentPresent = value.CurrentPresent == 1;
        PreviousAffinity = value.PreviousAffinity; CurrentAffinity = value.CurrentAffinity;
        Evaluation = DirectWitnessOutcome.ConvertEvaluation(value.Evaluation);
        PreviousPad = DirectWitnessOutcome.ConvertPad(value.PreviousPad); CurrentPad = DirectWitnessOutcome.ConvertPad(value.CurrentPad);
        MemoryKind = (MemoryKind)value.MemoryKind; PreviousMemoryKind = (MemoryKind)value.PreviousMemoryKind;
        CurrentMemoryKind = (MemoryKind)value.CurrentMemoryKind; Salience = value.Salience;
        PreviousTick = value.PreviousTick; CurrentTick = value.CurrentTick;
    }
}
