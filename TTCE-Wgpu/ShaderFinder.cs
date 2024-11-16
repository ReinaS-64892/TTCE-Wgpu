using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using net.rs64.TexTransCore;

namespace net.rs64.TexTransCoreEngineForWgpu
{
    public static class ShaderFinder
    {
        public static ShaderDictionary RegisterShaders(this TTCEWgpuDevice device, IEnumerable<string> shaderPaths)
        {
            var shaderDicts = new Dictionary<TTComputeType, Dictionary<string, TTComputeShaderID>>();
            foreach (var path in shaderPaths)
            {
                var computeName = Path.GetFileNameWithoutExtension(path);

                var srcText = File.ReadAllText(path);
                if (srcText.Contains("UnityCG.cginc")) { throw new InvalidDataException(" UnityCG.cginc は使用してはいけません！"); }

                var descriptions = TTComputeShaderUtility.Parse(srcText);
                if (descriptions is null) { continue; }

                if (shaderDicts.ContainsKey(descriptions.ComputeType) is false) { shaderDicts[descriptions.ComputeType] = new(); }

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
                            shaderDicts[descriptions.ComputeType][blendKey] = device.RegisterComputeShaderFromHLSL(path, csCode);

                            break;
                        }
                }
            }

            return new(shaderDicts);
        }

        public static IEnumerable<string> GetAllShaderPathWithCurrentDirectory() => GetAllShaderPath(Directory.GetCurrentDirectory());
        public static IEnumerable<string> GetAllShaderPath(string rootPath)
        {
            return Directory.GetFiles(rootPath, "*.ttcomp", SearchOption.AllDirectories).Concat(Directory.GetFiles(Directory.GetCurrentDirectory(), "*.ttblend", SearchOption.AllDirectories));
        }

        public class ShaderDictionary : ITexTransStandardComputeKey, ITexTransComputeKeyDictionary<string>, ITexTransComputeKeyDictionary<ITTBlendKey>
        {
            private Dictionary<TTComputeType, Dictionary<string, TTComputeShaderID>> _shaderDict;


            public ITTComputeKey this[string key] => _shaderDict[TTComputeType.GrabBlend][key];

            public ITTComputeKey this[ITTBlendKey key] => ((BlendKey)key).ComputeKey;

            public ITTComputeKey AlphaFill { get; private set; }
            public ITTComputeKey AlphaCopy { get; private set; }
            public ITTComputeKey AlphaMultiply { get; private set; }
            public ITTComputeKey AlphaMultiplyWithTexture { get; private set; }
            public ITTComputeKey ColorFill { get; private set; }
            public ITTComputeKey ColorMultiply { get; private set; }
            public ITTComputeKey BilinearReScaling { get; private set; }
            public ITTComputeKey GammaToLinear { get; private set; }
            public ITTComputeKey LinearToGamma { get; private set; }

            public ITTComputeKey Swizzling { get; private set; }

            public ITTBlendKey QueryBlendKey(string blendKeyName)
            {
                return new BlendKey(_shaderDict[TTComputeType.Blending][blendKeyName]);
            }

            public ITexTransComputeKeyDictionary<string> GenealCompute { get; private set; }

            public ShaderDictionary(Dictionary<TTComputeType, Dictionary<string, TTComputeShaderID>> dict)
            {
                _shaderDict = dict;
                AlphaFill = _shaderDict[TTComputeType.General][nameof(AlphaFill)];
                AlphaCopy = _shaderDict[TTComputeType.General][nameof(AlphaCopy)];
                AlphaMultiply = _shaderDict[TTComputeType.General][nameof(AlphaMultiply)];
                AlphaMultiplyWithTexture = _shaderDict[TTComputeType.General][nameof(AlphaMultiplyWithTexture)];
                ColorFill = _shaderDict[TTComputeType.General][nameof(ColorFill)];
                ColorMultiply = _shaderDict[TTComputeType.General][nameof(ColorMultiply)];
                BilinearReScaling = _shaderDict[TTComputeType.General][nameof(BilinearReScaling)];
                GammaToLinear = _shaderDict[TTComputeType.General][nameof(GammaToLinear)];
                LinearToGamma = _shaderDict[TTComputeType.General][nameof(LinearToGamma)];
                Swizzling = _shaderDict[TTComputeType.General][nameof(Swizzling)];
                GenealCompute = new GeneralComputeObject(_shaderDict[TTComputeType.General]);
            }

            class GeneralComputeObject : ITexTransComputeKeyDictionary<string>
            {
                private Dictionary<string, TTComputeShaderID> dictionary;

                public GeneralComputeObject(Dictionary<string, TTComputeShaderID> dictionary)
                {
                    this.dictionary = dictionary;
                }

                public ITTComputeKey this[string key] => dictionary[key];
            }
        }

        class BlendKey : ITTBlendKey
        {
            public ITTComputeKey ComputeKey;

            public BlendKey(ITTComputeKey computeKey)
            {
                ComputeKey = computeKey;
            }
        }
    }
}
