using System.Runtime.InteropServices;
using net.rs64.TexTransCore;
namespace net.rs64.TexTransCoreEngineForWgpu;

public sealed class TTCEWgpuDevice : IDisposable
{
    TexTransCoreEngineDeviceHandler _handler;
    internal HashSet<TTCEWgpu> _contexts;
    public bool AllowShaderCreation => _contexts.Count == 0;
    public TTCEWgpuDevice()
    {
        _handler = TexTransCoreEngineDeviceHandler.Create();
        _contexts = new();

        RegisterFormatConvertor();
    }

    private void RegisterFormatConvertor()
    {
        if (_handler.IsInvalid) { throw new ObjectDisposedException("TexTransCoreEngineDeviceHandler is dropped"); }
        if (AllowShaderCreation is false) { throw new InvalidOperationException("shader creation is not allowed"); }

        unsafe
        {
            NativeMethod.register_format_convertor((void*)_handler.DangerousGetHandle());
        }
    }
    public TTComputeShaderID RegisterComputeShaderFromHLSL(string hlslPath, string? hlslSource = null)
    {
        if (_handler.IsInvalid) { throw new ObjectDisposedException("TexTransCoreEngineDeviceHandler is dropped"); }
        if (AllowShaderCreation is false) { throw new InvalidOperationException("shader creation is not allowed"); }

        if (hlslSource is not null)
            unsafe
            {
                fixed (char* pathPtr = hlslPath)
                fixed (char* sourcePtr = hlslSource)
                {
                    var id = NativeMethod.register_compute_shader_from_hlsl((void*)_handler.DangerousGetHandle(), (ushort*)pathPtr, hlslPath.Length, (ushort*)sourcePtr, hlslSource.Length);
                    return new TTComputeShaderID(id);
                }
            }
        else
            unsafe
            {
                fixed (char* pathPtr = hlslPath)
                {
                    var id = NativeMethod.register_compute_shader_from_hlsl((void*)_handler.DangerousGetHandle(), (ushort*)pathPtr, hlslPath.Length, (ushort*)IntPtr.Zero, 0);
                    return new TTComputeShaderID(id);
                }
            }
    }

    public TTCE GetContext<TTCE>() where TTCE : TTCEWgpu, new()
    {
        if (_handler.IsInvalid) { throw new ObjectDisposedException("TexTransCoreEngineDeviceHandler is dropped"); }
        unsafe
        {
            var ptr = new IntPtr(NativeMethod.get_ttce_context((void*)_handler.DangerousGetHandle()));

            var ctx = new TTCE();
            ctx.NativeInitialize(this, new TexTransCoreEngineContextHandler(ptr));
            _contexts.Add(ctx);
            return ctx;
        }
    }


    public void Dispose()
    {
        if (_handler != null && _handler.IsInvalid is false)
        {
            foreach (var ctx in _contexts.ToArray()) { ctx.Dispose(); }
            _handler.Dispose();
        }
        GC.SuppressFinalize(this);
    }
}


class TexTransCoreEngineDeviceHandler : SafeHandle
{
    unsafe public static TexTransCoreEngineDeviceHandler Create() { return new TexTransCoreEngineDeviceHandler(new IntPtr(NativeMethod.create_tex_trans_engine_device())); }
    public TexTransCoreEngineDeviceHandler(IntPtr handle) : base(IntPtr.Zero, true)
    {
        SetHandle(handle);
    }

    public override bool IsInvalid => handle == IntPtr.Zero;

    protected override bool ReleaseHandle()
    {
        unsafe { NativeMethod.drop_tex_trans_engine_device((void*)handle); }
        return true;
    }
}

public struct TTComputeShaderID : ITTComputeKey
{
    uint _id;
    internal TTComputeShaderID(uint id)
    {
        _id = id;
    }
    internal uint GetID() => _id;

    public override string ToString()
    {
        return "TTComputeShaderID:" + _id;
    }
}
