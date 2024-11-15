using System;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.InteropServices;
using net.rs64.TexTransCore;
using ChannelFFI = net.rs64.TexTransCoreEngineForWgpu.TexTransCoreTextureChannel;

namespace net.rs64.TexTransCoreEngineForWgpu
{

    public class TTCEWgpu : ITexTransCreateTexture
    , ITexTransCopyRenderTexture
    , ITexTransGetComputeHandler
    , ITexTransRenderTextureIO
    , IDisposable
    {
        TTCEWgpuDevice _device = null!;
        TexTransCoreEngineContextHandler _handler = null!;

        internal HashSet<TTRenderTexture> _renderTextures = new();
        internal HashSet<TTComputeHandler> _computeHandlers = new();


        internal void NativeInitialize(TTCEWgpuDevice device, TexTransCoreEngineContextHandler handler)
        {
            _device = device;
            _handler = handler;
        }


        public TTRenderTexture GetRenderTexture(uint width, uint height, TexTransCore.TexTransCoreTextureChannel channel = TexTransCore.TexTransCoreTextureChannel.RGBA)
        {
            if (width == 0 || height == 0) { throw new ArgumentException(); }
            if (_handler.IsInvalid) { throw new ObjectDisposedException("TexTransCoreEngineContextHandler is dropped"); }

            unsafe
            {
                var ptr = new IntPtr(NativeMethod.get_render_texture((void*)_handler.DangerousGetHandle(), width, height, (ChannelFFI)channel));
                var rt = new TTRenderTexture(this, new TTRenderTextureHandler(ptr), channel);
                _renderTextures.Add(rt);
                return rt;
            }
        }
        public TTComputeHandler GetTTComputeHandler(TTComputeShaderID computeShaderID)
        {
            if (_handler.IsInvalid) { throw new ObjectDisposedException("TexTransCoreEngineContextHandler is dropped"); }

            unsafe
            {
                var ptr = new IntPtr(NativeMethod.get_compute_handler((void*)_handler.DangerousGetHandle(), computeShaderID.GetID()));
                var ttCH = new TTComputeHandler(this, new TTComputeHandlerPtrHandler(ptr));
                _computeHandlers.Add(ttCH);
                return ttCH;
            }
        }


        public void CopyTexture(TTRenderTexture dist, TTRenderTexture src)
        {
            if (_handler.IsInvalid) { throw new ObjectDisposedException("TexTransCoreEngineContextHandler is dropped"); }

            unsafe
            {
                NativeMethod.copy_texture((void*)_handler.DangerousGetHandle(), (void*)dist.GetPtr(), (void*)src.GetPtr());
            }
        }

        public void UploadTexture<T>(TTRenderTexture dist, ReadOnlySpan<T> dataSource, TexTransCore.TexTransCoreTextureFormat format) where T : unmanaged
        {
            if (_handler.IsInvalid) { throw new ObjectDisposedException("TexTransCoreEngineContextHandler is dropped"); }

            unsafe
            {
                fixed (T* ptr = dataSource)
                {
                    NativeMethod.upload_texture((void*)_handler.DangerousGetHandle(), (void*)dist.GetPtr(), (byte*)ptr, dataSource.Length * sizeof(T), (TexTransCoreTextureFormat)format);
                }
            }
        }

        public void DownloadTexture<T>(Span<T> dataDist, TexTransCore.TexTransCoreTextureFormat format, TTRenderTexture source) where T : unmanaged
        {
            if (_handler.IsInvalid) { throw new ObjectDisposedException("TexTransCoreEngineContextHandler is dropped"); }

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
        public ITTRenderTexture CreateRenderTexture(int width, int height, TexTransCore.TexTransCoreTextureChannel channel = TexTransCore.TexTransCoreTextureChannel.RGBA)
        {
            return GetRenderTexture((uint)width, (uint)height, channel);
        }

        public void CopyRenderTexture(ITTRenderTexture target, ITTRenderTexture source)
        {
            CopyTexture(target.Unwrap(), source.Unwrap());
        }

        public ITTComputeHandler GetComputeHandler(ITTComputeKey computeKey) { return GetTTComputeHandler(computeKey.Unwrap()); }

        public void Dispose()
        {
            if (_handler != null && _handler.IsInvalid is false)
            {
                foreach (var rt in _renderTextures.ToArray()) { rt.Dispose(); }
                foreach (var ch in _computeHandlers.ToArray()) { ch.Dispose(); }
                _device._contexts.Remove(this);

                _handler.Dispose();
            }
            GC.SuppressFinalize(this);
        }

        public void UploadTexture<T>(ITTRenderTexture uploadTarget, ReadOnlySpan<T> bytes, TexTransCore.TexTransCoreTextureFormat format) where T : unmanaged
        {
            UploadTexture((TTRenderTexture)uploadTarget, bytes, format);
        }

        public void DownloadTexture<T>(Span<T> dataDist, TexTransCore.TexTransCoreTextureFormat format, ITTRenderTexture renderTexture) where T : unmanaged
        {
            DownloadTexture(dataDist, format, (TTRenderTexture)renderTexture);
        }
    }

    internal static class TTCEWgpuEngineUtil
    {
        public static TTRenderTexture Unwrap(this ITTRenderTexture renderTexture) => (TTRenderTexture)renderTexture;
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
