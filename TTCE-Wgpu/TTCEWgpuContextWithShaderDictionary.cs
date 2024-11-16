
using net.rs64.TexTransCore;

namespace net.rs64.TexTransCoreEngineForWgpu
{
    public class TTCEWgpuContextWithShaderDictionary : TTCEWgpuContextBase, ITexTransComputeKeyQuery
    {
        public ShaderFinder.ShaderDictionary ShaderDictionary = null!;//気を付けるようにね！
        public ITexTransStandardComputeKey StandardComputeKey => ShaderDictionary;

        public ITexTransComputeKeyDictionary<string> GrabBlend => ShaderDictionary;

        public ITexTransComputeKeyDictionary<ITTBlendKey> BlendKey => ShaderDictionary;

        public ITexTransComputeKeyDictionary<string> GenealCompute => ShaderDictionary.GenealCompute;
    }
}
