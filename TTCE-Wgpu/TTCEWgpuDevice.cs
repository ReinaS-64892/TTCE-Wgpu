using System;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.InteropServices;
using System.Runtime.Serialization;
using net.rs64.TexTransCore;
namespace net.rs64.TexTransCoreEngineForWgpu
{

    public sealed class TTCEWgpuDevice : IDisposable
    {
        TexTransCoreEngineDeviceHandler? _handler;
        private bool _isDisposed;
        internal HashSet<TTCEWgpuContextBase> _contexts;
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
            if (_handler is null) { throw new ObjectDisposedException("TexTransCoreEngineDeviceHandler is dropped"); }
            if (AllowShaderCreation is false) { throw new InvalidOperationException("shader creation is not allowed"); }

            unsafe
            {
                NativeMethod.register_format_convertor((void*)_handler.DangerousGetHandle());
            }
        }
        public void SetDefaultTextureFormat(TexTransCore.TexTransCoreTextureFormat format)
        {
            if (_handler is null) { throw new ObjectDisposedException("TexTransCoreEngineDeviceHandler is dropped"); }
            if (AllowShaderCreation is false) { throw new InvalidOperationException("shader creation is not allowed"); }

            unsafe
            {
                NativeMethod.set_default_texture_format((void*)_handler.DangerousGetHandle(), (TexTransCoreTextureFormat)format);
            }
        }
        public TTComputeShaderID RegisterComputeShaderFromHLSL(string hlslPath, string? hlslSource = null)
        {
            if (_handler is null) { throw new ObjectDisposedException("TexTransCoreEngineDeviceHandler is dropped"); }
            if (AllowShaderCreation is false) { throw new InvalidOperationException("shader creation is not allowed"); }

            if (hlslSource is not null)
                unsafe
                {
                    fixed (char* pathPtr = hlslPath)
                    fixed (char* sourcePtr = hlslSource)
                    {
                        var idResult = NativeMethod.register_compute_shader_from_hlsl((void*)_handler.DangerousGetHandle(), (ushort*)pathPtr, hlslPath.Length, (ushort*)sourcePtr, hlslSource.Length);
                        if (idResult.result is false) { throw new Exception("register hlsl failed!, Please see log! \nSourceHLSLPath:" + hlslPath + "\nHLSLSource\n" + hlslSource); }
                        return new TTComputeShaderID(idResult.compute_shader_id);
                    }
                }
            else
                unsafe
                {
                    fixed (char* pathPtr = hlslPath)
                    {
                        var idResult = NativeMethod.register_compute_shader_from_hlsl((void*)_handler.DangerousGetHandle(), (ushort*)pathPtr, hlslPath.Length, (ushort*)IntPtr.Zero, 0);
                        if (idResult.result is false) { throw new Exception("register hlsl failed!, Please see log! \nSourceHLSLPath:" + hlslPath + "\nSource is file original text"); }
                        return new TTComputeShaderID(idResult.compute_shader_id);
                    }
                }
        }

        public TTCE GetContext<TTCE>() where TTCE : TTCEWgpuContextBase, new()
        {
            if (_handler is null) { throw new ObjectDisposedException("TexTransCoreEngineDeviceHandler is dropped"); }
            unsafe
            {
                var ptr = new IntPtr(NativeMethod.get_ttce_context((void*)_handler.DangerousGetHandle()));

                var ctx = new TTCE();
                ctx.NativeInitialize(this, new TexTransCoreEngineContextHandler(ptr));
                _contexts.Add(ctx);
                return ctx;
            }
        }


        void Dispose(bool disposing)
        {
            if (_isDisposed) { return; }

            if (disposing)
            {
                foreach (var ctx in _contexts.ToArray()) { ctx.Dispose(); }
                _handler?.Dispose();
                _handler = null;
            }

            _isDisposed = true;
        }
        public void Dispose()
        {
            Dispose(true);
            GC.SuppressFinalize(this);
        }

    }

    public static class TTCEWgpuRustCoreDebug
    {
        public static event Action<string>? DebugLog = null;

        static DLCallDelegate? log;

        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        unsafe delegate void DLCallDelegate(ushort* ptr, int len);
        unsafe static void DLCall(ushort* ptr, int len)
        {
            ReadOnlySpan<char> span = new ReadOnlySpan<char>(ptr, len);
            DebugLog?.Invoke(new string(span));
        }
        public static unsafe void LogHandlerInitialize()
        {
            log ??= new(DLCall);
            NativeMethod.set_debug_log_pointer((delegate* unmanaged[Cdecl]<ushort*, int, void>)Marshal.GetFunctionPointerForDelegate(log));
        }
        public static unsafe void LogHandlerDeInitialize()
        {
            NativeMethod.set_debug_log_pointer(null);
        }
    }
    public class TTCEWgpuRCDebugPrintToConsole : IDisposable
    {
        static int s_count = 0;
        bool _isDisposed = false;
        public TTCEWgpuRCDebugPrintToConsole()
        {
            if (s_count is 0)
            {
                TTCEWgpuRustCoreDebug.DebugLog += Console.WriteLine;
                TTCEWgpuRustCoreDebug.LogHandlerInitialize();
                _isDisposed = false;
            }
            s_count += 1;
        }
        public void Dispose()
        {
            if (_isDisposed) { return; }
            s_count -= 1;
            if (s_count is 0)
            {
                TTCEWgpuRustCoreDebug.DebugLog -= Console.WriteLine;
                TTCEWgpuRustCoreDebug.LogHandlerInitialize();
            }
            _isDisposed = true;
        }
    }

    class TexTransCoreEngineDeviceHandler : SafeHandle
    {
        unsafe public static TexTransCoreEngineDeviceHandler Create(TTCEWgpuDevice.RequestDevicePreference preference) { return new TexTransCoreEngineDeviceHandler(new IntPtr(NativeMethod.create_tex_trans_core_engine_device((RequestDevicePreference)preference))); }
        public TexTransCoreEngineDeviceHandler(IntPtr handle) : base(IntPtr.Zero, true)
        {
            SetHandle(handle);
        }

        public override bool IsInvalid => handle == IntPtr.Zero;

        protected override bool ReleaseHandle()
        {
            unsafe { NativeMethod.drop_tex_trans_core_engine_device((void*)handle); }
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




    [Serializable]
    internal class TTCEWgpuNativeError : Exception
    {
        public TTCEWgpuNativeError()
        {
        }

        public TTCEWgpuNativeError(string message) : base(message)
        {
        }

        public TTCEWgpuNativeError(string message, Exception innerException) : base(message, innerException)
        {
        }

        protected TTCEWgpuNativeError(SerializationInfo info, StreamingContext context) : base(info, context)
        {
        }
    }

}
