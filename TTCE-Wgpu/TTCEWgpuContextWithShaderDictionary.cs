
using net.rs64.TexTransCore;

namespace net.rs64.TexTransCoreEngineForWgpu
{
    public class TTCEWgpuContextWithShaderDictionary : TTCEWgpuContextBase, ITexTransComputeKeyQuery
    {
        public ShaderFinder.ShaderDictionary ShaderDictionary = null!;//気を付けるようにね！
        public ITexTransStandardComputeKey StandardComputeKey => ShaderDictionary;
        public TExKeyQ GetExKeyQuery<TExKeyQ>() where TExKeyQ : ITTExtraComputeKeyQuery
        {
            if(ShaderDictionary is not TExKeyQ exKeyQ)  { throw new ComputeKeyInterfaceIsNotImplementException($"{GetType().Name} is not supported {typeof(TExKeyQ).GetType().Name}."); }
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
