using System;
using System.Runtime.InteropServices;
using System.Runtime.Serialization;
using net.rs64.TexTransCore;
namespace net.rs64.TexTransCoreEngineForWgpu
{

    public sealed class TTComputeHandler : IDisposable, ITTComputeHandler
    {
        TTCEWgpuContextBase _engineContext;
        TTComputeHandlerPtrHandler? _handler;
        private bool _isDisposed = false;

        internal TTComputeHandler(TTCEWgpuContextBase engineContext, TTComputeHandlerPtrHandler handle)
        {
            _engineContext = engineContext;
            _handler = handle;
        }

        public int NameToID(string name)
        {
            if (_handler is null) { throw new ObjectDisposedException("TTComputeHandlerPtrHandler is dropped"); }
            unsafe
            {
                fixed (char* namePtr = name)
                {
                    var result = NativeMethod.get_bind_index((void*)_handler.DangerousGetHandle(), (ushort*)namePtr, name.Length);

                    if (result.result is false) { throw new ArgumentException(); }

                    return (int)result.bind_index;
                }
            }
        }

        public void SetRenderTexture(int nameID, TTRenderTexture renderTexture)
        {
            if (_handler is null) { throw new ObjectDisposedException("TTComputeHandlerPtrHandler is dropped"); }

            bool result;
            unsafe
            {
                result = NativeMethod.set_render_texture((void*)_handler.DangerousGetHandle(), (uint)nameID, (void*)renderTexture.GetPtr());
            }
            if (result is false)
            {
                throw new TTCEWgpuNativeError("Buffer upload failed! please see log!");
            }
        }

        public void UploadConstantsBuffer<T>(int nameID, ReadOnlySpan<T> buffer) where T : unmanaged
        { UploadBufferImpl(nameID, buffer, true); }
        public void UploadStorageBuffer<T>(int nameID, ReadOnlySpan<T> buffer) where T : unmanaged
        { UploadBufferImpl(nameID, buffer, false); }
        private void UploadBufferImpl<T>(int nameID, ReadOnlySpan<T> buffer, bool isConstants) where T : unmanaged
        {

            if (_handler is null) { throw new ObjectDisposedException("TTComputeHandlerPtrHandler is dropped"); }

            bool result;
            unsafe
            {
                fixed (T* bufferPtr = buffer)
                {
                    if (isConstants) result = NativeMethod.upload_constants_buffer((void*)_handler.DangerousGetHandle(), (uint)nameID, (byte*)bufferPtr, buffer.Length * sizeof(T));
                    else result = NativeMethod.upload_storage_buffer((void*)_handler.DangerousGetHandle(), (uint)nameID, (byte*)bufferPtr, buffer.Length * sizeof(T));
                }
            }
            if (result is false)
            {
                throw new TTCEWgpuNativeError("Buffer upload failed! please see log!");
            }
        }

        public (uint x, uint y, uint z) GetWorkGroupSize()
        {
            if (_handler is null) { throw new ObjectDisposedException("TTComputeHandlerPtrHandler is dropped"); }
            unsafe
            {
                var wgs = NativeMethod.get_work_group_size((void*)_handler.DangerousGetHandle());
                return (wgs.x, wgs.y, wgs.z);
            }
        }
        public (uint x, uint y, uint z) WorkGroupSize => GetWorkGroupSize();
        public void Dispatch(uint x, uint y, uint z)
        {
            if (_handler is null) { throw new ObjectDisposedException("TTComputeHandlerPtrHandler is dropped"); }

            unsafe
            {
                NativeMethod.dispatch((void*)_handler.DangerousGetHandle(), x, y, z);
            }
        }


        public void Dispatch(int x, int y, int z) { Dispatch(x, y, z); }


        public void SetTexture(int id, ITTRenderTexture tex)
        {
            if (tex is not TTRenderTexture rt) { throw new InvalidCastException(); }
            SetRenderTexture(id, rt);
        }

        void Dispose(bool disposing)
        {
            if (_isDisposed) { return; }

            if (disposing)
            {
                _engineContext._computeHandlers.Remove(this);
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
    class TTComputeHandlerPtrHandler : SafeHandle
    {
        public TTComputeHandlerPtrHandler(IntPtr handle) : base(IntPtr.Zero, true)
        {
            SetHandle(handle);
        }

        public override bool IsInvalid => handle == IntPtr.Zero;

        protected override bool ReleaseHandle()
        {
            unsafe { NativeMethod.drop_compute_handler((void*)handle); }
            return true;
        }
    }
}
