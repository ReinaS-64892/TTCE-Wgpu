using System;
using System.Runtime.InteropServices;
using net.rs64.TexTransCore;
namespace net.rs64.TexTransCoreEngineForWgpu
{

    public sealed class TTComputeHandler : IDisposable, ITTComputeHandler
    {
        TTCEWgpu _engineContext;
        TTComputeHandlerPtrHandler _handler;


        internal TTComputeHandler(TTCEWgpu engineContext, TTComputeHandlerPtrHandler handle)
        {
            _engineContext = engineContext;
            _handler = handle;
        }

        public int NameToID(string name)
        {
            if (_handler.IsInvalid) { throw new ObjectDisposedException("TTComputeHandlerPtrHandler is dropped"); }
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
            if (_handler.IsInvalid) { throw new ObjectDisposedException("TTComputeHandlerPtrHandler is dropped"); }

            unsafe
            {
                NativeMethod.set_render_texture((void*)_handler.DangerousGetHandle(), (uint)nameID, (void*)renderTexture.GetPtr());
            }
        }

        public void UploadConstantsBuffer<T>(int nameID, ReadOnlySpan<T> buffer) where T : unmanaged
        { UploadBufferImpl(nameID, buffer, true); }
        public void UploadStorageBuffer<T>(int nameID, ReadOnlySpan<T> buffer) where T : unmanaged
        { UploadBufferImpl(nameID, buffer, false); }
        private void UploadBufferImpl<T>(int nameID, ReadOnlySpan<T> buffer, bool isConstants) where T : unmanaged
        {

            if (_handler.IsInvalid) { throw new ObjectDisposedException("TTComputeHandlerPtrHandler is dropped"); }

            unsafe
            {
                fixed (T* bufferPtr = buffer)
                {
                    if (isConstants) NativeMethod.upload_constants_buffer((void*)_handler.DangerousGetHandle(), (uint)nameID, (byte*)bufferPtr, buffer.Length * sizeof(T));
                    else NativeMethod.upload_storage_buffer((void*)_handler.DangerousGetHandle(), (uint)nameID, (byte*)bufferPtr, buffer.Length * sizeof(T));
                }
            }
        }

        public (uint x, uint y, uint z) GetWorkGroupSize()
        {
            if (_handler.IsInvalid) { throw new ObjectDisposedException("TTComputeHandlerPtrHandler is dropped"); }
            unsafe
            {
                var wgs = NativeMethod.get_work_group_size((void*)_handler.DangerousGetHandle());
                return (wgs.x, wgs.y, wgs.z);
            }
        }
        public (uint x, uint y, uint z) WorkGroupSize => GetWorkGroupSize();
        public void Dispatch(uint x, uint y, uint z)
        {
            if (_handler.IsInvalid) { throw new ObjectDisposedException("TTComputeHandlerPtrHandler is dropped"); }

            unsafe
            {
                NativeMethod.dispatch((void*)_handler.DangerousGetHandle(), x, y, z);
            }
        }

        public void Dispose()
        {
            if (_handler != null && _handler.IsInvalid is false)
            {
                _engineContext._computeHandlers.Remove(this);
                _handler.Dispose();
            }
            GC.SuppressFinalize(this);
        }

        public void Dispatch(int x, int y, int z) { Dispatch(x, y, z); }


        public void SetTexture(int id, ITTRenderTexture tex)
        {
            if (tex is not TTRenderTexture rt) { throw new InvalidCastException(); }
            SetRenderTexture(id, rt);
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
