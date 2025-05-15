using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using net.rs64.TexTransCore;

namespace net.rs64.TexTransCoreEngineForWgpu
{
    public static class ShaderFinder
    {
        const string INCLUDE_SAMPLER_TEMPLATE_LIEN = "#include \"SamplerTemplate.hlsl\"";
        public static ShaderDictionary RegisterShadersWithCurrentDirectory(this TTCEWgpuDevice device)
        {
            return RegisterShaders(device, GetAllShaderPathWithCurrentDirectory(), CurrentDirectoryFind);
        }
        public static ShaderDictionary RegisterShaders(this TTCEWgpuDevice device, IEnumerable<string> shaderPaths, Func<string, string> findTemplate)
        {
            var shaderDicts = new Dictionary<TTComputeType, Dictionary<string, TTComputeShaderID>>();
            var specialShaderDicts = new Dictionary<TTComputeType, Dictionary<string, ISpecialComputeKey>>();
            foreach (var path in shaderPaths)
            {
                var computeName = Path.GetFileNameWithoutExtension(path);

                var srcText = File.ReadAllText(path);
                if (srcText.Contains("UnityCG.cginc")) { throw new InvalidDataException(" UnityCG.cginc は使用してはいけません！"); }

                var descriptions = TTComputeShaderUtility.Parse(srcText);
                if (descriptions is null) { continue; }

                if (shaderDicts.ContainsKey(descriptions.ComputeType) is false) { shaderDicts[descriptions.ComputeType] = new(); }
                if (specialShaderDicts.ContainsKey(descriptions.ComputeType) is false) { specialShaderDicts[descriptions.ComputeType] = new(); }

                switch (descriptions.ComputeType)
                {
                    case TTComputeType.General:
                        {
                            shaderDicts[descriptions.ComputeType][computeName] = device.RegisterComputeShaderFromHLSL(path);
                            break;
                        }
                    case TTComputeType.GrabBlend:
                        {
                            shaderDicts[descriptions.ComputeType][computeName] = device.RegisterComputeShaderFromHLSL(path);
                            break;
                        }
                    case TTComputeType.Blending:
                        {
                            var blendKey = descriptions["Key"];
                            var csCode = srcText + TTComputeShaderUtility.BlendingShaderTemplate;
                            specialShaderDicts[descriptions.ComputeType][blendKey] = new BlendKey(device.RegisterComputeShaderFromHLSL(path, csCode));
                            break;
                        }
                    case TTComputeType.Sampler:
                        {
                            var csCodeR = findTemplate("TextureResizingTemplate.hlsl").Replace(INCLUDE_SAMPLER_TEMPLATE_LIEN, srcText);
                            var csCodeT = findTemplate("TransSamplingTemplate.hlsl").Replace(INCLUDE_SAMPLER_TEMPLATE_LIEN, srcText);
                            var csCodeA = findTemplate("AtlasSamplingTemplate.hlsl").Replace(INCLUDE_SAMPLER_TEMPLATE_LIEN, srcText);
                            var resizingKey = device.RegisterComputeShaderFromHLSL(path, csCodeR);
                            var transSamplerKey = device.RegisterComputeShaderFromHLSL(path, csCodeT);
                            var atlasSamplerKey = device.RegisterComputeShaderFromHLSL(path, csCodeA);
                            specialShaderDicts[descriptions.ComputeType][computeName] = new SamplerKey(resizingKey, transSamplerKey, atlasSamplerKey);
                            break;
                        }
                }
            }

            return new(shaderDicts, specialShaderDicts);
        }

        public static Func<string, string> CurrentDirectoryFind = str => ShaderFinder.FindTextAsset(Directory.GetCurrentDirectory(), str);

        public static IEnumerable<string> GetAllShaderPathWithCurrentDirectory() => GetAllShaderPath(Directory.GetCurrentDirectory());
        public static IEnumerable<string> GetAllShaderPath(string rootPath)
        {
            return Directory.GetFiles(rootPath, "*.ttcomp", SearchOption.AllDirectories).Concat(Directory.GetFiles(Directory.GetCurrentDirectory(), "*.ttblend", SearchOption.AllDirectories));
        }

        public static string FindTextAsset(string rootPath, string fileName)
        {
            var candidates = Directory.GetFiles(rootPath, fileName, SearchOption.AllDirectories);
            return File.ReadAllText(candidates.First(s => (s.Contains("Tex") && s.Contains("Trans")) || (s.Contains("tex") && s.Contains("trans")) || s.Contains("TTCE")));
        }
        public class ShaderDictionary : ITexTransStandardComputeKey
        , ITransTextureComputeKey
        , IQuayGeneraleComputeKey
        , IBlendingComputeKey
        , ISamplerComputeKey
        , INearTransComputeKey
        , IAtlasComputeKey
        , IAtlasSamplerComputeKey
        {
            private Dictionary<TTComputeType, Dictionary<string, TTComputeShaderID>> _shaderDict;
            private Dictionary<TTComputeType, Dictionary<string, ISpecialComputeKey>> _specialShaderDict;



            public ITTComputeKey AlphaFill { get; private set; }
            public ITTComputeKey AlphaCopy { get; private set; }
            public ITTComputeKey AlphaMultiply { get; private set; }
            public ITTComputeKey AlphaMultiplyWithTexture { get; private set; }
            public ITTComputeKey ColorFill { get; private set; }
            public ITTComputeKey ColorMultiply { get; private set; }
            public ITTComputeKey GammaToLinear { get; private set; }
            public ITTComputeKey LinearToGamma { get; private set; }

            public ITTComputeKey Swizzling { get; private set; }
            public ITTSamplerKey DefaultSampler { get; private set; }

            public ITexTransComputeKeyDictionary<string> GenealCompute { get; private set; }

            public IKeyValueStore<string, ITTBlendKey> QueryBlendKey { get; private set; }
            public ITexTransComputeKeyDictionary<ITTBlendKey> BlendKey { get; private set; }

            public ITexTransComputeKeyDictionary<string> GrabBlend { get; private set; }

            public IKeyValueStore<string, ITTSamplerKey> SamplerKey { get; private set; }
            public ITexTransComputeKeyDictionary<ITTSamplerKey> ResizingSamplerKey { get; private set; }
            public ITexTransComputeKeyDictionary<ITTSamplerKey> TransSamplerKey { get; private set; }

            public ITTComputeKey FillR { get; private set; }
            public ITTComputeKey FillRG { get; private set; }
            public ITTComputeKey FillROnly { get; private set; }
            public ITTComputeKey FillGOnly { get; private set; }

            public ITTComputeKey TransMapping { get; private set; }
            public ITTComputeKey TransMappingWithDepth { get; private set; }

            public ITTComputeKey TransWarpNone { get; private set; }
            public ITTComputeKey TransWarpStretch { get; private set; }

            public ITTComputeKey DepthRenderer { get; private set; }
            public ITTComputeKey CullingDepth { get; private set; }

            public ITTComputeKey NearTransTexture { get; private set; }
            public ITTComputeKey PositionMapper { get; private set; }
            public ITTComputeKey FilleFloat4StorageBuffer { get; private set; }
            public ITTComputeKey NearDistanceFadeWrite { get; private set; }
            public ITTComputeKey RectangleTransMapping { get; private set; }
            public ITTComputeKey MergeAtlasedTextures { get; private set; }

            public ITexTransComputeKeyDictionary<ITTSamplerKey> AtlasSamplerKey { get; private set; }

            public ShaderDictionary(Dictionary<TTComputeType, Dictionary<string, TTComputeShaderID>> dict, Dictionary<TTComputeType, Dictionary<string, ISpecialComputeKey>> specialDicts)
            {
                _shaderDict = dict;
                _specialShaderDict = specialDicts;
                AlphaFill = _shaderDict[TTComputeType.General][nameof(AlphaFill)];
                AlphaCopy = _shaderDict[TTComputeType.General][nameof(AlphaCopy)];
                AlphaMultiply = _shaderDict[TTComputeType.General][nameof(AlphaMultiply)];
                AlphaMultiplyWithTexture = _shaderDict[TTComputeType.General][nameof(AlphaMultiplyWithTexture)];
                ColorFill = _shaderDict[TTComputeType.General][nameof(ColorFill)];
                ColorMultiply = _shaderDict[TTComputeType.General][nameof(ColorMultiply)];
                GammaToLinear = _shaderDict[TTComputeType.General][nameof(GammaToLinear)];
                LinearToGamma = _shaderDict[TTComputeType.General][nameof(LinearToGamma)];
                Swizzling = _shaderDict[TTComputeType.General][nameof(Swizzling)];
                FillR = _shaderDict[TTComputeType.General][nameof(FillR)];
                FillRG = _shaderDict[TTComputeType.General][nameof(FillRG)];
                FillROnly = _shaderDict[TTComputeType.General][nameof(FillROnly)];
                FillGOnly = _shaderDict[TTComputeType.General][nameof(FillGOnly)];

                TransMapping = _shaderDict[TTComputeType.General][nameof(TransMapping)];
                TransMappingWithDepth = _shaderDict[TTComputeType.General][nameof(TransMappingWithDepth)];

                TransWarpNone = _shaderDict[TTComputeType.General][nameof(TransWarpNone)];
                TransWarpStretch = _shaderDict[TTComputeType.General][nameof(TransWarpStretch)];
                DepthRenderer = _shaderDict[TTComputeType.General][nameof(DepthRenderer)];
                CullingDepth = _shaderDict[TTComputeType.General][nameof(CullingDepth)];

                GenealCompute = new Str2Dict(_shaderDict[TTComputeType.General]);
                GrabBlend = new Str2Dict(_shaderDict[TTComputeType.GrabBlend]);

                QueryBlendKey = new BlendKeyQuery(_specialShaderDict[TTComputeType.Blending]);
                BlendKey = new BlendKeyToComputeKey();

                SamplerKey = new SamplerKeyQuery(_specialShaderDict[TTComputeType.Sampler]);
                ResizingSamplerKey = new SamplerToResizeSamplerKey();
                TransSamplerKey = new SamplerToTransSamplerKey();

                DefaultSampler = SamplerKey["AverageSampling"];

                NearTransTexture = _shaderDict[TTComputeType.General][nameof(NearTransTexture)];
                PositionMapper = _shaderDict[TTComputeType.General][nameof(PositionMapper)];
                FilleFloat4StorageBuffer = _shaderDict[TTComputeType.General][nameof(FilleFloat4StorageBuffer)];
                NearDistanceFadeWrite = _shaderDict[TTComputeType.General][nameof(NearDistanceFadeWrite)];
                RectangleTransMapping = _shaderDict[TTComputeType.General][nameof(RectangleTransMapping)];
                MergeAtlasedTextures = _shaderDict[TTComputeType.General][nameof(MergeAtlasedTextures)];

                AtlasSamplerKey = new SamplerToAtlasSamplerKey();
            }

            class Str2Dict : ITexTransComputeKeyDictionary<string>
            {
                private Dictionary<string, TTComputeShaderID> dictionary;

                public Str2Dict(Dictionary<string, TTComputeShaderID> dictionary)
                {
                    this.dictionary = dictionary;
                }

                public ITTComputeKey this[string key] => dictionary[key];
            }
            class BlendKeyQuery : IKeyValueStore<string, ITTBlendKey>
            {
                private Dictionary<string, ISpecialComputeKey> dictionary;

                public BlendKeyQuery(Dictionary<string, ISpecialComputeKey> dictionary)
                {
                    this.dictionary = dictionary;
                }

                public ITTBlendKey this[string key] => (ITTBlendKey)dictionary[key];
            }
            class BlendKeyToComputeKey : ITexTransComputeKeyDictionary<ITTBlendKey>
            {
                public ITTComputeKey this[ITTBlendKey key] => ((BlendKey)key).ComputeKey;
            }
            class SamplerKeyQuery : IKeyValueStore<string, ITTSamplerKey>
            {
                private Dictionary<string, ISpecialComputeKey> dictionary;

                public SamplerKeyQuery(Dictionary<string, ISpecialComputeKey> dictionary)
                {
                    this.dictionary = dictionary;
                }

                public ITTSamplerKey this[string key] => (ITTSamplerKey)dictionary[key];
            }
            class SamplerToResizeSamplerKey : ITexTransComputeKeyDictionary<ITTSamplerKey>
            {
                public ITTComputeKey this[ITTSamplerKey key] => ((SamplerKey)key).ResizingComputeKey;
            }
            class SamplerToTransSamplerKey : ITexTransComputeKeyDictionary<ITTSamplerKey>
            {
                public ITTComputeKey this[ITTSamplerKey key] => ((SamplerKey)key).TransSamplerComputeKey;
            }
            class SamplerToAtlasSamplerKey : ITexTransComputeKeyDictionary<ITTSamplerKey>
            {
                public ITTComputeKey this[ITTSamplerKey key] => ((SamplerKey)key).AtlasSamplerComputeKey;
            }
        }

        class BlendKey : ITTBlendKey, ISpecialComputeKey
        {
            public ITTComputeKey ComputeKey;

            public BlendKey(ITTComputeKey computeKey)
            {
                ComputeKey = computeKey;
            }
        }
        class SamplerKey : ITTSamplerKey, ISpecialComputeKey
        {
            public ITTComputeKey ResizingComputeKey;
            public ITTComputeKey TransSamplerComputeKey;
            public ITTComputeKey AtlasSamplerComputeKey;

            public SamplerKey(ITTComputeKey resizingKey, ITTComputeKey transSamplerKey, ITTComputeKey atlasSamplerKey)
            {
                ResizingComputeKey = resizingKey;
                TransSamplerComputeKey = transSamplerKey;
                AtlasSamplerComputeKey = atlasSamplerKey;
            }
        }

        public interface ISpecialComputeKey
        {

        }
    }
}
