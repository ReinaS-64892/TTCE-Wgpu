using System;
using System.Runtime.InteropServices;
using net.rs64.TexTransCore;
namespace net.rs64.TexTransCoreEngineForWgpu
{
    public sealed class TTWgpuStorageBuffer : IDisposable, ITTStorageBuffer
    {
        TTCEWgpuContextBase _engineContext;
        TTStorageBufferHandler? _handler;
        private bool _isDisposed;
        public event Action<TTWgpuStorageBuffer>? DisposeCall;
        internal readonly bool _downloadable;

        public string Name { get; set; } = "Wgpu-TTStorageBuffer";
        internal TTWgpuStorageBuffer(TTCEWgpuContextBase engineContext, TTStorageBufferHandler handler, bool downloadable)
        {
            _engineContext = engineContext;
            _handler = handler;
            _downloadable = downloadable;
        }

        public void Dispose(bool disposing)
        {
            if (_isDisposed) { return; }

            if (disposing)
            {
                _engineContext._storageBuffers.Remove(this);
                _handler?.Dispose();
                _handler = null;
                DisposeCall?.Invoke(this);
            }

            _isDisposed = true;
        }
        public void Dispose()
        {
            Dispose(true);
            GC.SuppressFinalize(this);
        }

        internal IntPtr GetPtr()
        {
            if (_handler is null) { throw new ObjectDisposedException("TTStorageBuffer is dropped"); }

            unsafe
            {
                return _handler.DangerousGetHandle();
            }

        }
    }
    class TTStorageBufferHandler : SafeHandle
    {
        public TTStorageBufferHandler(IntPtr handle) : base(IntPtr.Zero, true)
        {
            SetHandle(handle);
        }

        public override bool IsInvalid => handle == IntPtr.Zero;

        protected override bool ReleaseHandle()
        {
            unsafe { NativeMethod.drop_storage_buffer((void*)handle); }
            return true;
        }
    }
}
