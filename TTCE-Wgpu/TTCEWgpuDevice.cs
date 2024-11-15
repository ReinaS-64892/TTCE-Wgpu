using System;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.InteropServices;
using net.rs64.TexTransCore;
namespace net.rs64.TexTransCoreEngineForWgpu
{

    public sealed class TTCEWgpuDevice : IDisposable
    {
        TexTransCoreEngineDeviceHandler _handler;
        internal HashSet<TTCEWgpu> _contexts;
        public bool AllowShaderCreation => _contexts.Count == 0;
        public TTCEWgpuDevice(RequestDevicePreference preference = RequestDevicePreference.Auto)
        {
            _handler = TexTransCoreEngineDeviceHandler.Create(preference);
            _contexts = new();

            RegisterFormatConvertor();
        }
        public enum RequestDevicePreference : uint
        {
            Auto,
            DiscreteGPU,
            IntegratedGPUOrCPU,
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
        public void SetDefaultTextureFormat(TexTransCore.TexTransCoreTextureFormat format)
        {
            if (_handler.IsInvalid) { throw new ObjectDisposedException("TexTransCoreEngineDeviceHandler is dropped"); }
            if (AllowShaderCreation is false) { throw new InvalidOperationException("shader creation is not allowed"); }

            unsafe
            {
                NativeMethod.set_default_texture_format((void*)_handler.DangerousGetHandle(), (TexTransCoreTextureFormat)format);
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

    public static class TTCEWgpuRustCoreDebug
    {
        public static event Action<string> DebugLog = null!;
        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        unsafe delegate void DLCallDelegate(ushort* ptr, int len);
        unsafe static void DLCall(ushort* ptr, int len)
        {
            ReadOnlySpan<char> span = new ReadOnlySpan<char>(ptr, len);
            DebugLog(new string(span));
        }
        public static unsafe void LogHandlerInitialize()
        {
            var ptr = (delegate* unmanaged[Cdecl]<ushort*, int, void>)Marshal.GetFunctionPointerForDelegate((DLCallDelegate)DLCall);
            NativeMethod.set_debug_log_pointer(ptr);
        }
        public static unsafe void LogHandlerDeInitialize()
        {
            NativeMethod.set_debug_log_pointer(null);
        }


    }


    class TexTransCoreEngineDeviceHandler : SafeHandle
    {
        unsafe public static TexTransCoreEngineDeviceHandler Create(TTCEWgpuDevice.RequestDevicePreference preference) { return new TexTransCoreEngineDeviceHandler(new IntPtr(NativeMethod.create_tex_trans_engine_device((RequestDevicePreference)preference))); }
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
}
