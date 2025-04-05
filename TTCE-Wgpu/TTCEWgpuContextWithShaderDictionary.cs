
using net.rs64.TexTransCore;

namespace net.rs64.TexTransCoreEngineForWgpu
{
    public class TTCEWgpuDeviceWidthShaderDictionary : TTCEWgpuDevice
    {
        protected readonly ShaderFinder.ShaderDictionary _shaderDictionary;
        public TTCEWgpuDeviceWidthShaderDictionary(
                RequestDevicePreference preference = RequestDevicePreference.Auto
                , TexTransCore.TexTransCoreTextureFormat format = TexTransCore.TexTransCoreTextureFormat.Byte
            ): base(preference)
        {
            SetDefaultTextureFormat(format);
            _shaderDictionary = this.RegisterShadersWithCurrentDirectory();
        }

        protected new TTCE CreateContext<TTCE>() where TTCE : TTCEWgpuContextWithShaderDictionary, new()
        {
            var ctx = base.CreateContext<TTCE>();
            ctx.i_ShaderDictionary = _shaderDictionary;
            return ctx;
        }
        public new TTCEWgpuContextWithShaderDictionary GetTTCEWgpuContext()
        {
            return CreateContext<TTCEWgpuContextWithShaderDictionary>();
        }
    }
    public class TTCEWgpuContextWithShaderDictionary : TTCEWgpuContextBase, ITexTransComputeKeyQuery
    {
        internal ShaderFinder.ShaderDictionary i_ShaderDictionary = null!;//気を付けるようにね！
        protected ShaderFinder.ShaderDictionary ShaderDictionary => i_ShaderDictionary;
        public ITexTransStandardComputeKey StandardComputeKey => ShaderDictionary;
        public TExKeyQ GetExKeyQuery<TExKeyQ>() where TExKeyQ : ITTExtraComputeKeyQuery
        {
            if (ShaderDictionary is not TExKeyQ exKeyQ) { throw new ComputeKeyInterfaceIsNotImplementException($"{GetType().Name} is not supported {typeof(TExKeyQ).GetType().Name}."); }
            return exKeyQ;
        }

        public ITexTransComputeKeyDictionary<string> GrabBlend => ShaderDictionary.GrabBlend;

        public ITexTransComputeKeyDictionary<ITTBlendKey> BlendKey => ShaderDictionary.BlendKey;

        public ITexTransComputeKeyDictionary<string> GenealCompute => ShaderDictionary.GenealCompute;

        public IKeyValueStore<string, ITTSamplerKey> SamplerKey => ShaderDictionary.SamplerKey;

        public ITexTransComputeKeyDictionary<ITTSamplerKey> ResizingSamplerKey => ShaderDictionary.ResizingSamplerKey;

        public ITexTransComputeKeyDictionary<ITTSamplerKey> TransSamplerKey => ShaderDictionary.TransSamplerKey;

    }
}
