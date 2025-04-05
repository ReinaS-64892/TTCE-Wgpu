using System;
using System.Runtime.InteropServices;
using net.rs64.TexTransCore;
namespace net.rs64.TexTransCoreEngineForWgpu
{
    public sealed class TTWgpuRenderTexture : IDisposable, ITTRenderTexture
    {
        TTCEWgpuContextBase _engineContext;
        TTRenderTextureHandler? _handler;
        private bool _isDisposed = false;
        public event Action<TTWgpuRenderTexture>? DisposeCall;
        TexTransCore.TexTransCoreTextureChannel _channel;

        string _name;

        public int Width => (int)GetWidth();

        public int Hight => (int)GetHeight();


        public string Name { get => _name; set => _name = value; }

        public TexTransCore.TexTransCoreTextureChannel ContainsChannel => _channel;

        internal TTWgpuRenderTexture(TTCEWgpuContextBase engineContext, TTRenderTextureHandler handle, TexTransCore.TexTransCoreTextureChannel channel)
        {
            _engineContext = engineContext;
            _handler = handle;
            _name = "TTRenderTexture-Wgpu";
            _channel = channel;
        }

        public uint GetWidth()
        {
            if (_handler is null) { throw new ObjectDisposedException("TTRenderTextureHandler is dropped"); }

            unsafe
            {
                return NativeMethod.get_width((void*)_handler.DangerousGetHandle());
            }
        }
        public uint GetHeight()
        {
            if (_handler is null) { throw new ObjectDisposedException("TTRenderTextureHandler is dropped"); }

            unsafe
            {
                return NativeMethod.get_height((void*)_handler.DangerousGetHandle());
            }
        }

        internal IntPtr GetPtr()
        {
            if (_handler is null) { throw new ObjectDisposedException("TTRenderTextureHandler is dropped"); }

            unsafe
            {
                return _handler.DangerousGetHandle();
            }

        }
        void Dispose(bool disposing)
        {
            if (_isDisposed) { return; }

            if (disposing)
            {
                _engineContext._renderTextures.Remove(this);
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

    }
    class TTRenderTextureHandler : SafeHandle
    {
        public TTRenderTextureHandler(IntPtr handle) : base(IntPtr.Zero, true)
        {
            SetHandle(handle);
        }

        public override bool IsInvalid => handle == IntPtr.Zero;

        protected override bool ReleaseHandle()
        {
            unsafe { NativeMethod.drop_render_texture((void*)handle); }
            return true;
        }
    }
}
