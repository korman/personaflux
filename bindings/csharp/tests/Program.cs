using System;
using System.IO;
using System.Runtime.InteropServices;
using PersonaFlux;
using PersonaFlux.Interop;

static class Program
{
    private static int Main()
    {
        var library = Environment.GetEnvironmentVariable("PERSONAFLUX_NATIVE_PATH");
        if (string.IsNullOrWhiteSpace(library))
        {
            Console.Error.WriteLine("PERSONAFLUX_NATIVE_PATH is required for the C# integration smoke test.");
            return 2;
        }

        NativeLibrary.SetDllImportResolver(typeof(Simulation).Assembly, (name, _, _) =>
            string.Equals(name, "personaflux", StringComparison.Ordinal) ? NativeLibrary.Load(Path.GetFullPath(library)) : IntPtr.Zero);

        Assert(Simulation.QueryApiVersion() == Simulation.AbiVersion, "ABI version");
        Assert(Simulation.QueryModelVersion() == Simulation.ModelVersion, "model version");
        Assert(Marshal.SizeOf<Native.Pad>() == 12, "PAD ABI layout");
        Assert(Marshal.SizeOf<Native.RelationshipLookup>() == 20, "relationship ABI layout");
        Assert(Marshal.SizeOf<Native.EvaluationResult>() == 36, "evaluation ABI layout");
        Assert(Marshal.SizeOf<Native.DirectWitnessDeed>() == 56, "deed ABI layout");
        Assert(Marshal.SizeOf<Native.MemoryRecord>() == 80, "memory ABI layout");
        Assert(Marshal.SizeOf<Native.DirectWitnessOutcome>() == 216, "outcome ABI layout");
        Assert(Marshal.SizeOf<Native.SubmissionResult>() == 248, "submission ABI layout");
        Assert(Marshal.SizeOf<Native.MemberState>() == 32, "member ABI layout");
        Assert(Marshal.SizeOf<Native.Event>() == 192, "event ABI layout");

        using var simulation = new Simulation(17);
        var faction = simulation.AddFaction("观察者");
        Assert(simulation.GetFactionName(faction) == "观察者", "UTF-8 faction name");
        var observer = simulation.AddMember(faction);
        var actor = simulation.AddMember(faction);
        var target = simulation.AddMember(faction);
        simulation.SetMemberRelationship(observer, target, 0.75f);
        var lookup = simulation.GetEffectiveMemberRelationship(observer, target);
        Assert(lookup.Present && Math.Abs(lookup.Affinity - 0.75f) < 1e-6f, "relationship lookup");

        simulation.ClearEvents();
        var result = simulation.SubmitDirectWitness(new DirectWitnessDeed(42, observer, actor, target, 0.8f, 0.1f, false));
        Assert(result.Kind == SubmissionKind.Applied && result.Outcome.HasValue, "applied submission");
        Assert(simulation.GetEvents().Count >= 2, "event production");
        Assert(simulation.GetMemory(observer, 42).HasValue, "memory query");
        var eventCountBeforeInvalidInput = simulation.GetEvents().Count;
        try
        {
            simulation.SetMemberRelationship(observer, target, float.NaN);
            throw new InvalidOperationException("NaN input was accepted");
        }
        catch (PersonaFluxException error) when (error.Code == ResultCode.InvalidArgument) { }
        Assert(simulation.GetEvents().Count == eventCountBeforeInvalidInput, "invalid input has no side effects");
        var duplicate = simulation.SubmitDirectWitness(new DirectWitnessDeed(42, observer, actor, target, -0.8f, 0.9f, false));
        Assert(duplicate.Kind == SubmissionKind.Duplicate, "deduplication");
        var batch = simulation.SubmitDirectWitnessBatch(new[] {
            new DirectWitnessDeed(43, observer, actor, target, 0.2f, 0.0f, false),
            new DirectWitnessDeed(43, observer, actor, target, -0.2f, 1.0f, false),
        });
        Assert(batch.Count == 2 && batch[0].Kind == SubmissionKind.Applied && batch[1].Kind == SubmissionKind.Duplicate, "batch order and deduplication");
        simulation.Step(60);
        Assert(simulation.CurrentTick == 60, "logical time");
        return 0;
    }

    private static void Assert(bool condition, string name)
    {
        if (!condition) throw new InvalidOperationException("Assertion failed: " + name);
    }
}
