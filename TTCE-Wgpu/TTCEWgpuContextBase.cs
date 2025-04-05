using System;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.InteropServices;
using net.rs64.TexTransCore;
using ChannelFFI = net.rs64.TexTransCoreEngineForWgpu.TexTransCoreTextureChannel;

namespace net.rs64.TexTransCoreEngineForWgpu
{
    public class TTCEWgpuContextBase : IDisposable
    , ITexTransCreateTexture
    , ITexTransCopyRenderTexture
    , ITexTransGetComputeHandler
    , ITexTransRenderTextureIO
    , ITexTransDriveStorageBufferHolder
    {
        protected TTCEWgpuDevice _device = null!;
        private TexTransCoreEngineContextHandler? _handler = null;
        private bool _isDisposed;


        internal HashSet<TTWgpuRenderTexture> _renderTextures = new();
        internal HashSet<TTWgpuComputeHandler> _computeHandlers = new();
        internal HashSet<TTWgpuStorageBuffer> _storageBuffers = new();


        internal void NativeInitialize(TTCEWgpuDevice device, TexTransCoreEngineContextHandler handler)
        {
            _device = device;
            _handler = handler;
        }


        public TTWgpuRenderTexture GetRenderTexture(uint width, uint height, TexTransCore.TexTransCoreTextureChannel channel = TexTransCore.TexTransCoreTextureChannel.RGBA)
        {
            if (width == 0 || height == 0) { throw new ArgumentException(); }
            if (_handler is null) { throw new ObjectDisposedException("TexTransCoreEngineContextHandler is dropped"); }

            unsafe
            {
                var ptr = new IntPtr(NativeMethod.get_render_texture((void*)_handler.DangerousGetHandle(), width, height, (ChannelFFI)channel));
                var rt = new TTWgpuRenderTexture(this, new TTRenderTextureHandler(ptr), channel);
                _renderTextures.Add(rt);
                return rt;
            }
        }
        public TTWgpuComputeHandler GetTTComputeHandler(TTComputeShaderID computeShaderID)
        {
            if (_handler is null) { throw new ObjectDisposedException("TexTransCoreEngineContextHandler is dropped"); }

            unsafe
            {
                var ptr = new IntPtr(NativeMethod.get_compute_handler((void*)_handler.DangerousGetHandle(), computeShaderID.GetID()));
                var ttCH = new TTWgpuComputeHandler(this, new TTComputeHandlerPtrHandler(ptr));
                _computeHandlers.Add(ttCH);
                return ttCH;
            }
        }


        public void CopyTexture(TTWgpuRenderTexture dist, TTWgpuRenderTexture src)
        {
            if (_handler is null) { throw new ObjectDisposedException("TexTransCoreEngineContextHandler is dropped"); }
            if (dist.EqualSize(src) is false) { throw new ArgumentException(); }

            unsafe
            {
                NativeMethod.copy_texture((void*)_handler.DangerousGetHandle(), (void*)dist.GetPtr(), (void*)src.GetPtr());
            }
        }

        public void UploadTexture<T>(TTWgpuRenderTexture dist, ReadOnlySpan<T> dataSource, TexTransCore.TexTransCoreTextureFormat format) where T : unmanaged
        {
            if (_handler is null) { throw new ObjectDisposedException("TexTransCoreEngineContextHandler is dropped"); }

            unsafe
            {
                fixed (T* ptr = dataSource)
                {
                    NativeMethod.upload_texture((void*)_handler.DangerousGetHandle(), (void*)dist.GetPtr(), (byte*)ptr, dataSource.Length * sizeof(T), (TexTransCoreTextureFormat)format);
                }
            }
        }

        public void DownloadTexture<T>(Span<T> dataDist, TexTransCore.TexTransCoreTextureFormat format, TTWgpuRenderTexture source) where T : unmanaged
        {
            if (_handler is null) { throw new ObjectDisposedException("TexTransCoreEngineContextHandler is dropped"); }
            if (source.GetWidth() < 64 || source.GetHeight() < 64) { throw new InvalidOperationException("Texture downloading of 64x64 or above are allowed."); }

            unsafe
            {
                var ptrLen = dataDist.Length * sizeof(T);
                var dataSize = source.GetWidth() * source.GetHeight() * EnginUtil.GetPixelParByte(format, source.ContainsChannel);
                if (ptrLen != dataSize) { throw new ArgumentOutOfRangeException(); }

                fixed (T* ptr = dataDist)
                {
                    NativeMethod.download_texture((void*)_handler.DangerousGetHandle(), (byte*)ptr, ptrLen, (TexTransCoreTextureFormat)format, (void*)source.GetPtr());
                }
            }
        }
        public TTWgpuStorageBuffer AllocateStorageBuffer<T>(int length, bool downloadable = false) where T : unmanaged
        {
            if (_handler is null) { throw new ObjectDisposedException("TexTransCoreEngineContextHandler is dropped"); }

            unsafe
            {
                var storageBufferPtr = new IntPtr(NativeMethod.allocate_storage_buffer((void*)_handler.DangerousGetHandle(), sizeof(T) * length, downloadable));
                var sb = new TTWgpuStorageBuffer(this, new TTStorageBufferHandler(storageBufferPtr), downloadable);
                _storageBuffers.Add(sb);
                return sb;
            }
        }

        public TTWgpuStorageBuffer UploadStorageBuffer<T>(ReadOnlySpan<T> data, bool downloadable = false) where T : unmanaged
        {
            if (_handler is null) { throw new ObjectDisposedException("TexTransCoreEngineContextHandler is dropped"); }

            unsafe
            {
                var dataLen = data.Length * sizeof(T);
                fixed (T* dataPtr = data)
                {

                    var storageBufferPtr = new IntPtr(NativeMethod.upload_storage_buffer((void*)_handler.DangerousGetHandle(), (byte*)dataPtr, dataLen, downloadable));
                    var sb = new TTWgpuStorageBuffer(this, new TTStorageBufferHandler(storageBufferPtr), downloadable);
                    _storageBuffers.Add(sb);
                    return sb;
                }
            }
        }

        public void DownloadBuffer<T>(Span<T> dist, TTWgpuStorageBuffer buffer) where T : unmanaged
        {
            if (_handler is null) { throw new ObjectDisposedException("TexTransCoreEngineContextHandler is dropped"); }
            if (buffer._downloadable is false) { throw new InvalidOperationException("This Storage buffer is not downloadable"); }
            unsafe
            {
                var dataLen = dist.Length * sizeof(T);
                using (buffer)
                    fixed (T* bufPtr = dist)
                    {
                        NativeMethod.download_storage_buffer((void*)_handler.DangerousGetHandle(), (byte*)bufPtr, dataLen, (void*)buffer.GetPtr());
                    }
            }
        }


        public ITTRenderTexture CreateRenderTexture(int width, int height, TexTransCore.TexTransCoreTextureChannel channel = TexTransCore.TexTransCoreTextureChannel.RGBA)
        {
            return GetRenderTexture((uint)width, (uint)height, channel);
        }

        public void CopyRenderTexture(ITTRenderTexture target, ITTRenderTexture source)
        {
            CopyTexture(target.Unwrap(), source.Unwrap());
        }

        public ITTComputeHandler GetComputeHandler(ITTComputeKey computeKey) { return GetTTComputeHandler(computeKey.Unwrap()); }


        public void UploadTexture<T>(ITTRenderTexture uploadTarget, ReadOnlySpan<T> bytes, TexTransCore.TexTransCoreTextureFormat format) where T : unmanaged
        {
            UploadTexture((TTWgpuRenderTexture)uploadTarget, bytes, format);
        }

        public void DownloadTexture<T>(Span<T> dataDist, TexTransCore.TexTransCoreTextureFormat format, ITTRenderTexture renderTexture) where T : unmanaged
        {
            DownloadTexture(dataDist, format, (TTWgpuRenderTexture)renderTexture);
        }
        ITTStorageBuffer ITexTransDriveStorageBufferHolder.AllocateStorageBuffer<T>(int length, bool downloadable)
        { return AllocateStorageBuffer<T>(length, downloadable); }
        ITTStorageBuffer ITexTransDriveStorageBufferHolder.UploadStorageBuffer<T>(ReadOnlySpan<T> data, bool downloadable)
        { return UploadStorageBuffer(data, downloadable); }
        public void DownloadBuffer<T>(Span<T> dist, ITTStorageBuffer takeToFrom) where T : unmanaged
        { DownloadBuffer(dist, (TTWgpuStorageBuffer)takeToFrom); }

        protected virtual void Dispose(bool disposing)
        {
            if (_isDisposed) { return; }

            if (disposing)
            {
                foreach (var rt in _renderTextures.ToArray()) { rt.Dispose(); }
                foreach (var ch in _computeHandlers.ToArray()) { ch.Dispose(); }
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

    internal static class TTCEWgpuEngineUtil
    {
        public static TTWgpuRenderTexture Unwrap(this ITTRenderTexture renderTexture) => (TTWgpuRenderTexture)renderTexture;
        public static TTComputeShaderID Unwrap(this ITTComputeKey computeKey) => (TTComputeShaderID)computeKey;

    }

    class TexTransCoreEngineContextHandler : SafeHandle
    {
        public TexTransCoreEngineContextHandler(IntPtr handle) : base(IntPtr.Zero, true)
        {
            SetHandle(handle);
        }

        public override bool IsInvalid => handle == IntPtr.Zero;

        protected override bool ReleaseHandle()
        {
            unsafe { NativeMethod.drop_ttce_context((void*)handle); }
            return true;
        }
    }
}
