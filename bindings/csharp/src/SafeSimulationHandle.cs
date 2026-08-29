using System;
using Microsoft.Win32.SafeHandles;

namespace PersonaFlux;

internal sealed class SafeSimulationHandle : SafeHandleZeroOrMinusOneIsInvalid
{
    private SafeSimulationHandle() : base(true) { }

    internal SafeSimulationHandle(IntPtr value) : base(true) => SetHandle(value);

    protected override bool ReleaseHandle()
    {
        Interop.Native.pf_simulation_destroy(handle);
        return true;
    }
}
