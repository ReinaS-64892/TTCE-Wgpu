using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using net.rs64.TexTransCore;

namespace net.rs64.TexTransCoreEngineForWgpu
{
    public static class ShaderFinder
    {
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
                            var csCode = findTemplate("TextureResizingTemplate.hlsl").Replace("//$$$SAMPLER_CODE$$$", srcText);
                            var resizingKey = device.RegisterComputeShaderFromHLSL(path, csCode);
                            specialShaderDicts[descriptions.ComputeType][computeName] = new SamplerKey(resizingKey);
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
            return File.ReadAllText(candidates.First(s => s.Contains("TexTransCore")));
        }
        public class ShaderDictionary : ITexTransStandardComputeKey
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

                GenealCompute = new Str2Dict(_shaderDict[TTComputeType.General]);
                GrabBlend = new Str2Dict(_shaderDict[TTComputeType.GrabBlend]);

                QueryBlendKey = new BlendKeyQuery(_specialShaderDict[TTComputeType.Blending]);
                BlendKey = new BlendKeyToComputeKey();

                SamplerKey = new SamplerKeyQuery(_specialShaderDict[TTComputeType.Sampler]);
                ResizingSamplerKey = new SamplerToResizeSamplerKey();


                DefaultSampler = SamplerKey["AverageSampling"];
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
            public SamplerKey(ITTComputeKey resizingKey)
            {
                ResizingComputeKey = resizingKey;
            }
        }

        public interface ISpecialComputeKey
        {

        }
    }
}
